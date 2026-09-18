using Avalonia.Controls;
using Avalonia.Interactivity;

namespace WhaleLink.Desktop;

public partial class MainWindow : Window
{
    private readonly DaemonPipeClient _daemon = new();
    private readonly ControlPlaneClient _controlPlane = new();
    private readonly EnrollmentCredentialStore _credentials = new();
    private IReadOnlyList<RoomSummary> _rooms = [];

    public MainWindow()
    {
        InitializeComponent();
        Opened += async (_, _) => await RefreshStatusAsync();
    }

    private async void CheckStatus(object? sender, RoutedEventArgs e) => await RefreshStatusAsync();

    private async void RedeemInvite(object? sender, RoutedEventArgs e)
    {
        try
        {
            if (!Uri.TryCreate(ServerUrl.Text, UriKind.Absolute, out var server)
                || !server.Scheme.Equals(Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("请输入有效的 HTTPS 服务器地址。");
            var enrollment = await _controlPlane.RedeemInviteAsync(server, InviteCode.Text ?? string.Empty);
            _credentials.Save(enrollment.RoomId, enrollment.DataPlaneConfig);
            InviteCode.Text = string.Empty;
            RoomId.Text = enrollment.RoomId;
            try
            {
                await _daemon.ImportEnrollmentAsync(enrollment.RoomId, enrollment.DataPlaneConfig);
                StatusText.Text = "已安全保存并导入本地守护进程。请从本地服务启动连接。";
            }
            catch (Exception)
            {
                StatusText.Text = "已安全保存入网材料；本地守护进程未运行时可稍后导入。";
            }
        }
        catch (Exception exception)
        {
            StatusText.Text = exception.Message;
        }
    }

    private void ImportInvitePayload(object? sender, RoutedEventArgs e)
    {
        try
        {
            var payload = InvitePayloadParser.Parse(InvitePayload.Text);
            if (payload.Server is not null) ServerUrl.Text = payload.Server.ToString().TrimEnd('/');
            InviteCode.Text = payload.Code;
            InvitePayload.Text = string.Empty;
            StatusText.Text = "邀请码已导入；请确认服务器地址后兑换。";
        }
        catch (Exception exception) { StatusText.Text = exception.Message; }
    }

    private async void ConnectRoom(object? sender, RoutedEventArgs e)
    {
        try
        {
            await _daemon.StartTunnelAsync(RoomId.Text ?? string.Empty);
            StatusText.Text = "本地守护进程正在启动连接。";
        }
        catch (Exception exception)
        {
            StatusText.Text = exception.Message;
        }
    }

    private async void DisconnectRoom(object? sender, RoutedEventArgs e)
    {
        try
        {
            await _daemon.StopTunnelAsync();
            StatusText.Text = "本地守护进程已停止连接。";
        }
        catch (Exception exception)
        {
            StatusText.Text = exception.Message;
        }
    }

    private async void ExportDiagnostics(object? sender, RoutedEventArgs e)
    {
        try { StatusText.Text = await _daemon.ExportDiagnosticsAsync(); }
        catch (Exception exception) { StatusText.Text = exception.Message; }
    }

    private async void LoadRooms(object? sender, RoutedEventArgs e)
    {
        try
        {
            _rooms = await _controlPlane.GetRoomsAsync(AdministratorServer(), AdministratorToken());
            AdminRoom.ItemsSource = _rooms;
            AdminRoom.SelectedIndex = _rooms.Count > 0 ? 0 : -1;
            AdminStatusText.Text = _rooms.Count == 0 ? "服务器尚未配置固定房间。" : $"已加载 {_rooms.Count} 个固定房间。";
        }
        catch (Exception exception) { AdminStatusText.Text = exception.Message; }
    }

    private void AdminRoomChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (SelectedRoom() is { } room)
            AdminStatusText.Text = $"{room.DisplayName}：{room.MemberCount} 个成员，凭据 epoch {room.CredentialEpoch}。";
    }

    private async void CreateInvite(object? sender, RoutedEventArgs e)
    {
        try
        {
            var lifetime = int.Parse(((ComboBoxItem)InviteLifetime.SelectedItem!).Tag!.ToString()!);
            var invite = await _controlPlane.CreateInviteAsync(AdministratorServer(), AdministratorToken(), SelectedRoom().Id, DateTimeOffset.UtcNow.AddMinutes(lifetime));
            IssuedInvite.Text = invite.Code;
            InviteIdToRevoke.Text = invite.Id;
            AdminStatusText.Text = $"邀请码已创建，有效至 {invite.ExpiresAt.LocalDateTime:G}。请通过受控渠道交付。";
        }
        catch (Exception exception) { AdminStatusText.Text = exception.Message; }
    }

    private async void RevokeInvite(object? sender, RoutedEventArgs e)
    {
        try
        {
            await _controlPlane.RevokeInviteAsync(AdministratorServer(), AdministratorToken(), InviteIdToRevoke.Text ?? string.Empty);
            IssuedInvite.Text = string.Empty;
            AdminStatusText.Text = "邀请码已撤销。";
        }
        catch (Exception exception) { AdminStatusText.Text = exception.Message; }
    }

    private async void LoadMembers(object? sender, RoutedEventArgs e)
    {
        try
        {
            var members = await _controlPlane.GetMembersAsync(AdministratorServer(), AdministratorToken(), SelectedRoom().Id);
            MembersText.Text = members.Count == 0
                ? "当前没有已登记成员。"
                : string.Join(Environment.NewLine, members.Select(member => $"{member.Id}  加入 {member.JoinedAt.LocalDateTime:G}  最近活动 {member.LastSeenAt.LocalDateTime:G}"));
            AdminStatusText.Text = $"已加载 {members.Count} 个成员。";
        }
        catch (Exception exception) { AdminStatusText.Text = exception.Message; }
    }

    private async void RotateCredentials(object? sender, RoutedEventArgs e)
    {
        try
        {
            var room = await _controlPlane.RotateCredentialsAsync(AdministratorServer(), AdministratorToken(), SelectedRoom().Id);
            _rooms = _rooms.Select(item => item.Id == room.Id ? room : item).ToArray();
            AdminRoom.ItemsSource = _rooms;
            AdminRoom.SelectedItem = _rooms.Single(item => item.Id == room.Id);
            AdminStatusText.Text = $"{room.DisplayName} 已轮换至凭据 epoch {room.CredentialEpoch}。";
        }
        catch (Exception exception) { AdminStatusText.Text = exception.Message; }
    }

    private Uri AdministratorServer()
    {
        if (!Uri.TryCreate(AdminServerUrl.Text, UriKind.Absolute, out var server))
            throw new ArgumentException("请输入有效的管理员 HTTPS 服务器地址。");
        if (!server.Scheme.Equals(Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("管理员服务器地址必须使用 HTTPS。");
        return server;
    }

    private string AdministratorToken()
        => string.IsNullOrWhiteSpace(AdminToken.Text) ? throw new ArgumentException("请输入管理员令牌。") : AdminToken.Text;

    private RoomSummary SelectedRoom()
        => AdminRoom.SelectedItem as RoomSummary ?? throw new InvalidOperationException("请先加载并选择一个固定房间。");

    private async Task RefreshStatusAsync()
    {
        StatusText.Text = await _daemon.GetHealthSummaryAsync();
    }
}

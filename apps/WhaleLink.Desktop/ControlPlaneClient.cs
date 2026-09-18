using System.Net.Http.Json;
using System.Text.Json;

namespace WhaleLink.Desktop;

internal sealed class ControlPlaneClient(HttpClient? httpClient = null)
{
    private readonly HttpClient _http = httpClient ?? new HttpClient { Timeout = TimeSpan.FromSeconds(15) };

    public async Task<EnrollmentResult> RedeemInviteAsync(Uri serverBaseUri, string inviteCode, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        if (string.IsNullOrWhiteSpace(inviteCode)) throw new ArgumentException("请输入邀请码。", nameof(inviteCode));
        var endpoint = new Uri(serverBaseUri, $"/api/v1/invites/{Uri.EscapeDataString(inviteCode)}/redeem");
        using var response = await _http.PostAsync(endpoint, content: null, cancellationToken);
        using var document = await ReadSuccessAsync(response, "邀请码兑换失败。", cancellationToken);
        var data = document.RootElement.GetProperty("data");
        var enrollment = data.GetProperty("enrollment");
        return new EnrollmentResult(
            data.GetProperty("room_id").GetString()!,
            enrollment.GetProperty("data_plane").GetRawText());
    }

    public async Task<IReadOnlyList<RoomSummary>> GetRoomsAsync(Uri serverBaseUri, string administratorToken, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        using var request = AuthorizedRequest(HttpMethod.Get, serverBaseUri, "/api/v1/rooms", administratorToken);
        using var response = await _http.SendAsync(request, cancellationToken);
        using var document = await ReadSuccessAsync(response, "无法读取固定房间。", cancellationToken);
        return document.RootElement.GetProperty("data").EnumerateArray()
            .Select(room => new RoomSummary(
                room.GetProperty("id").GetString()!,
                room.GetProperty("display_name").GetString()!,
                room.GetProperty("member_count").GetInt32(),
                room.GetProperty("credential_epoch").GetInt64()))
            .ToArray();
    }

    public async Task<InviteIssued> CreateInviteAsync(Uri serverBaseUri, string administratorToken, string roomId, DateTimeOffset expiresAt, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        using var request = AuthorizedRequest(HttpMethod.Post, serverBaseUri, $"/api/v1/rooms/{Uri.EscapeDataString(roomId)}/invites", administratorToken);
        request.Content = JsonContent.Create(new { expires_at = expiresAt.UtcDateTime });
        using var response = await _http.SendAsync(request, cancellationToken);
        using var document = await ReadSuccessAsync(response, "无法创建邀请码。", cancellationToken);
        var data = document.RootElement.GetProperty("data");
        return new InviteIssued(
            data.GetProperty("id").GetString()!,
            data.GetProperty("code").GetString()!,
            data.GetProperty("expires_at").GetDateTimeOffset());
    }

    public async Task RevokeInviteAsync(Uri serverBaseUri, string administratorToken, string inviteId, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        using var request = AuthorizedRequest(HttpMethod.Delete, serverBaseUri, $"/api/v1/invites/{Uri.EscapeDataString(inviteId)}", administratorToken);
        using var response = await _http.SendAsync(request, cancellationToken);
        using var _ = await ReadSuccessAsync(response, "无法撤销邀请码。", cancellationToken);
    }

    public async Task<IReadOnlyList<MemberSummary>> GetMembersAsync(Uri serverBaseUri, string administratorToken, string roomId, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        using var request = AuthorizedRequest(HttpMethod.Get, serverBaseUri, $"/api/v1/rooms/{Uri.EscapeDataString(roomId)}/members", administratorToken);
        using var response = await _http.SendAsync(request, cancellationToken);
        using var document = await ReadSuccessAsync(response, "无法读取成员状态。", cancellationToken);
        return document.RootElement.GetProperty("data").EnumerateArray()
            .Select(member => new MemberSummary(
                member.GetProperty("id").GetString()!,
                member.GetProperty("joined_at").GetDateTimeOffset(),
                member.GetProperty("last_seen_at").GetDateTimeOffset()))
            .ToArray();
    }

    public async Task<RoomSummary> RotateCredentialsAsync(Uri serverBaseUri, string administratorToken, string roomId, CancellationToken cancellationToken = default)
    {
        EnsureHttps(serverBaseUri);
        using var request = AuthorizedRequest(HttpMethod.Post, serverBaseUri, $"/api/v1/rooms/{Uri.EscapeDataString(roomId)}/credentials/rotate", administratorToken);
        request.Content = new ByteArrayContent([]);
        using var response = await _http.SendAsync(request, cancellationToken);
        using var document = await ReadSuccessAsync(response, "无法轮换房间凭据。", cancellationToken);
        var data = document.RootElement.GetProperty("data");
        return new RoomSummary(
            data.GetProperty("id").GetString()!,
            data.GetProperty("display_name").GetString()!,
            data.GetProperty("member_count").GetInt32(),
            data.GetProperty("credential_epoch").GetInt64());
    }

    private static HttpRequestMessage AuthorizedRequest(HttpMethod method, Uri serverBaseUri, string path, string administratorToken)
    {
        if (string.IsNullOrWhiteSpace(administratorToken)) throw new ArgumentException("请输入管理员令牌。", nameof(administratorToken));
        var request = new HttpRequestMessage(method, new Uri(serverBaseUri, path));
        request.Headers.Add("X-WhaleLink-Token", administratorToken);
        return request;
    }

    private static void EnsureHttps(Uri serverBaseUri)
    {
        if (!serverBaseUri.Scheme.Equals(Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("控制面地址必须使用 HTTPS。", nameof(serverBaseUri));
    }

    private static async Task<JsonDocument> ReadSuccessAsync(HttpResponseMessage response, string fallbackMessage, CancellationToken cancellationToken)
    {
        var document = await JsonDocument.ParseAsync(await response.Content.ReadAsStreamAsync(cancellationToken), cancellationToken: cancellationToken);
        if (response.IsSuccessStatusCode && document.RootElement.TryGetProperty("ok", out var ok) && ok.GetBoolean()) return document;
        var message = document.RootElement.TryGetProperty("error", out var error) && error.TryGetProperty("message", out var text)
            ? text.GetString() : fallbackMessage;
        document.Dispose();
        throw new InvalidOperationException(message ?? fallbackMessage);
    }
}

internal sealed record EnrollmentResult(string RoomId, string DataPlaneConfig);
internal sealed record RoomSummary(string Id, string DisplayName, int MemberCount, long CredentialEpoch)
{
    public override string ToString() => $"{DisplayName} ({Id})";
}
internal sealed record InviteIssued(string Id, string Code, DateTimeOffset ExpiresAt);
internal sealed record MemberSummary(string Id, DateTimeOffset JoinedAt, DateTimeOffset LastSeenAt);

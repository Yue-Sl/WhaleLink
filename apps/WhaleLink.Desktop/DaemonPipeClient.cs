using System.IO.Pipes;
using System.Text;
using System.Text.Json;

namespace WhaleLink.Desktop;

/// <summary>Client for the versioned, user-scoped <c>WhaleLink.v1</c> Named Pipe.</summary>
internal sealed class DaemonPipeClient
{
    private const string PipeName = "WhaleLink.v1";

    public async Task<string> GetHealthSummaryAsync()
    {
        try
        {
            var response = await SendRequestAsync("health", new { });
            return string.IsNullOrWhiteSpace(response) ? "守护进程没有返回状态。" : "守护进程已响应。";
        }
        catch (TimeoutException) { return "守护进程未运行或尚未响应。"; }
        catch (IOException) { return "无法连接本地守护进程。"; }
    }

    public async Task ImportEnrollmentAsync(string roomId, string enrollmentJson)
    {
        using var document = JsonDocument.Parse(enrollmentJson);
        var response = await SendRequestAsync("enrollment.import", new { room_id = roomId, enrollment = document.RootElement });
        using var result = JsonDocument.Parse(response);
        if (!result.RootElement.TryGetProperty("ok", out var ok) || !ok.GetBoolean())
            throw new InvalidOperationException("本地守护进程未能导入入网材料。");
    }

    public async Task StartTunnelAsync(string roomId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(roomId);
        await EnsureSuccessAsync("tunnel.start", new { room_id = roomId }, "本地守护进程未能启动连接。");
    }

    public async Task StopTunnelAsync()
        => await EnsureSuccessAsync("tunnel.stop", new { }, "本地守护进程未能停止连接。");

    public async Task<string> ExportDiagnosticsAsync()
    {
        var response = await SendRequestAsync("diagnostics.export", new { });
        using var result = JsonDocument.Parse(response);
        if (!result.RootElement.TryGetProperty("ok", out var ok) || !ok.GetBoolean())
            throw new InvalidOperationException("本地守护进程未能导出诊断摘要。");
        var data = result.RootElement.GetProperty("data");
        var status = data.TryGetProperty("data_plane_status", out var value) ? value.GetString() : "unknown";
        return $"诊断摘要：数据面状态 {status}；不含凭据或网络地址。";
    }

    private static async Task EnsureSuccessAsync(string method, object parameters, string errorMessage)
    {
        var response = await SendRequestAsync(method, parameters);
        using var result = JsonDocument.Parse(response);
        if (!result.RootElement.TryGetProperty("ok", out var ok) || !ok.GetBoolean())
            throw new InvalidOperationException(errorMessage);
    }

    private static async Task<string> SendRequestAsync(string method, object parameters)
    {
        await using var pipe = new NamedPipeClientStream(".", PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);
        await pipe.ConnectAsync(500);
        var request = JsonSerializer.Serialize(new { protocol_version = 1, request_id = Guid.NewGuid(), method, @params = parameters });
        await pipe.WriteAsync(Encoding.UTF8.GetBytes(request + "\n"));
        await pipe.FlushAsync();
        using var reader = new StreamReader(pipe, Encoding.UTF8, leaveOpen: true);
        return await reader.ReadLineAsync() ?? string.Empty;
    }
}

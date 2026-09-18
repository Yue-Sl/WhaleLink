namespace WhaleLink.Desktop;

/// <summary>
/// Parses an invite code or a decoded WhaleLink QR payload. Parsing is local
/// only: it never contacts the control plane and never persists the payload.
/// </summary>
internal static class InvitePayloadParser
{
    public static InvitePayload Parse(string? payload)
    {
        var value = payload?.Trim();
        if (string.IsNullOrWhiteSpace(value)) throw new ArgumentException("请输入邀请码或二维码内容。", nameof(payload));
        if (!Uri.TryCreate(value, UriKind.Absolute, out var uri))
            return new InvitePayload(null, value);

        if (!uri.Scheme.Equals("whalelink", StringComparison.OrdinalIgnoreCase)
            || !uri.Host.Equals("invite", StringComparison.OrdinalIgnoreCase))
            return new InvitePayload(null, value);

        var parameters = ParseQuery(uri.Query);
        if (!parameters.TryGetValue("server", out var server)
            || !Uri.TryCreate(server, UriKind.Absolute, out var serverUri)
            || !serverUri.Scheme.Equals(Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("二维码缺少有效的控制面地址。", nameof(payload));
        if (!parameters.TryGetValue("code", out var code) || string.IsNullOrWhiteSpace(code))
            throw new ArgumentException("二维码缺少邀请码。", nameof(payload));
        return new InvitePayload(serverUri, code);
    }

    private static Dictionary<string, string> ParseQuery(string query)
    {
        return query.TrimStart('?')
            .Split('&', StringSplitOptions.RemoveEmptyEntries)
            .Select(part => part.Split('=', 2))
            .Where(pair => pair.Length == 2)
            .ToDictionary(
                pair => Uri.UnescapeDataString(pair[0]),
                pair => Uri.UnescapeDataString(pair[1]),
                StringComparer.OrdinalIgnoreCase);
    }
}

internal sealed record InvitePayload(Uri? Server, string Code);

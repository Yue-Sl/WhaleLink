using System.Security.AccessControl;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text;

namespace WhaleLink.Desktop;

/// <summary>Stores data-plane enrollment material for the current Windows user only.</summary>
internal sealed class EnrollmentCredentialStore
{
    private readonly string _directory;

    public EnrollmentCredentialStore(string? directory = null)
    {
        _directory = directory ?? Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "WhaleLink", "credentials");
    }

    public void Save(string roomId, string enrollmentMaterial)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(roomId);
        var path = PathFor(roomId);
        EnsureCurrentUserOnlyDirectory();
        var protectedBytes = ProtectedData.Protect(
            Encoding.UTF8.GetBytes(enrollmentMaterial),
            optionalEntropy: null,
            DataProtectionScope.CurrentUser);
        var temporary = path + ".tmp";
        File.WriteAllBytes(temporary, protectedBytes);
        File.Move(temporary, path, overwrite: true);
    }

    public bool TryLoad(string roomId, out string? enrollmentMaterial)
    {
        var path = PathFor(roomId);
        if (!File.Exists(path))
        {
            enrollmentMaterial = null;
            return false;
        }
        var plaintext = ProtectedData.Unprotect(
            File.ReadAllBytes(path),
            optionalEntropy: null,
            DataProtectionScope.CurrentUser);
        enrollmentMaterial = Encoding.UTF8.GetString(plaintext);
        return true;
    }

    private string PathFor(string roomId)
    {
        var hash = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(roomId))).ToLowerInvariant();
        return Path.Combine(_directory, $"{hash}.bin");
    }

    private void EnsureCurrentUserOnlyDirectory()
    {
        Directory.CreateDirectory(_directory);
        var currentUser = WindowsIdentity.GetCurrent().User
            ?? throw new InvalidOperationException("无法确定当前 Windows 用户。");
        var security = new DirectorySecurity();
        security.SetAccessRuleProtection(isProtected: true, preserveInheritance: false);
        security.SetOwner(currentUser);
        security.AddAccessRule(new FileSystemAccessRule(
            currentUser,
            FileSystemRights.FullControl,
            InheritanceFlags.ContainerInherit | InheritanceFlags.ObjectInherit,
            PropagationFlags.None,
            AccessControlType.Allow));
        new DirectoryInfo(_directory).SetAccessControl(security);
    }
}

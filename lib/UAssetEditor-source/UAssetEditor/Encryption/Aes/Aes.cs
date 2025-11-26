using System.Security.Cryptography;
using AesProvider = System.Security.Cryptography.Aes;

namespace UAssetEditor.Encryption.Aes;

// https://github.com/FabianFG/CUE4Parse/blob/0b9616c806ba53c112cf2805ad3f9f823dbb35d6/CUE4Parse/Encryption/Aes/Aes.cs
public static class Aes
{
    public const int ALIGN = 16;
    public const int BLOCK_SIZE = 16 * 8;
    
    private static readonly AesProvider Provider;

    public static byte[] Decrypt(this byte[] encrypted, FAesKey key)
    {
        return Provider.CreateDecryptor(key.Key, null).TransformFinalBlock(encrypted, 0, encrypted.Length);
    }

    public static byte[] Decrypt(this byte[] encrypted, int beginOffset, int count, FAesKey key)
    {
        return Provider.CreateDecryptor(key.Key, null).TransformFinalBlock(encrypted, beginOffset, count);
    }

    static Aes()
    {
        Provider = AesProvider.Create();
        Provider.Mode = CipherMode.ECB;
        Provider.Padding = PaddingMode.None;
        Provider.BlockSize = BLOCK_SIZE;
    }
}
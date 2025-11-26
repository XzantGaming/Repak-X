using System.Text;
using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Names;


public class FString
{
    public string Text;

    public FString(string str)
    {
        Text = str;
    }

    public static string Read(Reader reader)
    {
        var length = reader.Read<int>();
        
        switch (length)
        {
            case 0:
                return string.Empty;
            case < 0:
            {
                length = -length;
                var bytes = reader.ReadBytes(length * 2);
                return Encoding.Unicode.GetString(bytes).TrimEnd('\0');
            }
            default:
                return Encoding.ASCII.GetString(reader.ReadBytes(length)).TrimEnd('\0');
        }
    }

    public static void Write(Writer writer, string text)
    {
        if (text.Length > 0)
        {
            var str = text + '\0';
            writer.Write(str.Length);
            writer.WriteString(str);
            return;
        }
        
        writer.Write(0); // Length
    }

    public override string ToString()
    {
        return Text;
    }
}
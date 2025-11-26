using System.Runtime.CompilerServices;
using System.Text;

namespace UAssetEditor.Binary;

public class Writer : BinaryWriter
{
    public Writer(byte[] data) : base(new MemoryStream(data))
    { }

    public Writer(Stream stream) : base(stream)
    { }
    
    public Writer(string file) : this(File.Open(file, FileMode.OpenOrCreate, FileAccess.Write, FileShare.ReadWrite))
    { }
    
    public Writer() : base(new MemoryStream())
    { }
    
    public long Position
    {
        get => BaseStream.Position;
        set => BaseStream.Position = value;
    }
    
    public long Length => BaseStream.Length;

    public void Write<T>(T value)
    {
        var buffer = new byte[Unsafe.SizeOf<T>()];
        Unsafe.WriteUnaligned(ref buffer[0], value);
        BaseStream.Write(buffer);
    }

    public void WriteByte(byte b) => WriteBytes([b]);
    
    public void WriteBytes(byte[] buffer) => BaseStream.Write(buffer);

    public void WriteString(string str) => BaseStream.Write(Encoding.ASCII.GetBytes(str));

    public void WriteArray<T>(IEnumerable<T> values)
    {
        foreach (var value in values)
            Write(value);
    }

    public void CopyTo(Writer dest)
    {
        Position = 0;
        BaseStream.CopyTo(dest.BaseStream);
    }

    public byte[] ToArray()
    {
        var buffer = new byte[Length];
        Position = 0;
        BaseStream.Read(buffer);
            
        return buffer;
    }
}
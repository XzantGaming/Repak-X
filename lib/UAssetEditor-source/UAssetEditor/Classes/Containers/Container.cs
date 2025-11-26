using System.Collections;

namespace UAssetEditor.Classes.Containers;

public abstract class Container<T> : IEnumerable<T>
{
    protected List<T> Items { get; }

    public Container(List<T> items)
    {
        Items = items;
    }

    public void Add(T? item)
    {
        if (item is not null)
            Items.Add(item);
    }

    public virtual int GetIndex(string str)
    {
        throw new NotImplementedException();
    }

    public void Remove(int index)
    {
        Items.RemoveAt(index);
    }
    
    public void Remove(T item)
    {
        Items.Remove(item);
    }

    public bool Contains(T i) => Items.Contains(i);

    public T this[int index]
    {
        get => Items[index];
        set => Items[index] = value;
    }

    public T this[uint index]
    {
        get => Items[(int)index];
        set => Items[(int)index] = value;
    }

    public int Length => Items.Count;
    
    public IEnumerator<T> GetEnumerator()
    {
        return Items.GetEnumerator();
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return GetEnumerator();
    }
}
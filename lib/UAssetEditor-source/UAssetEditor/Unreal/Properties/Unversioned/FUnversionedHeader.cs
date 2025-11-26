using System.Collections;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Unversioned;

public class FUnversionedHeader
{
    public readonly IEnumerator<FFragment> Fragments;
    public readonly BitArray? ZeroMask;
    public readonly int ZeroMaskNum;
    public readonly int UnmaskedNum;
    public readonly bool HasNonZeroValues;

    public bool HasValues => HasNonZeroValues | ZeroMask?.Length > 0;
    
    public FUnversionedHeader(Reader asset)
    {
        var frags = new List<FFragment>();
        FFragment fragment; 
	    
        do
        {
            fragment = new FFragment(asset.Read<ushort>());
            frags.Add(fragment);

            if (fragment.bHasAnyZeroes)
                ZeroMaskNum += fragment.ValueNum;
            else
                UnmaskedNum += fragment.ValueNum;
        } while (!fragment.bIsLast);

        Fragments = frags.GetEnumerator();
	    
        if (ZeroMaskNum > 0)
        {
            switch (ZeroMaskNum)
            {
                case <= 8:
                    ZeroMask = new BitArray(asset.ReadBytes(1));
                    break;
                case <= 16:
                    ZeroMask = new BitArray(asset.ReadBytes(2));
                    break;
                default:
                {
                    var num = (ZeroMaskNum + 32 - 1) / 32;
                    ZeroMask = new BitArray(asset.ReadArray<int>(num));
                    break;
                }
            }
		    
            ZeroMask.Length = ZeroMaskNum;
        }

        // Check if we have any non-zero values
        if (UnmaskedNum > 0)
        {
            HasNonZeroValues = true;
            return;
        }

        if (ZeroMask == null) 
            return;
        
        foreach (bool bit in ZeroMask)
        {
            if (bit)
                continue;

            HasNonZeroValues = true;
            break;
        }
    }

    public static void Serialize(Writer writer, UStruct struc, List<UProperty> properties)
    {
        if (!properties.Any())
        {
            writer.Write(new FFragment
            {
                bIsLast = true
            }.Pack());
            return;
        }
        
        var frags = new List<FFragment>();
        var zeroMask = new List<bool>();
        
        var enumerator = properties.GetEnumerator();
        enumerator.MoveNext();
        
        // Add first frag
        AddFrag();
        
        // Make fragments
        foreach (var property in struc.Properties)
        {
            // TODO calculate when it should be zero
            // var isZero = property.IsZero;
            var isZero = false;

            if (property.Name == enumerator.Current.Name)
            {
                IncludeProperty(isZero);
                
                if (enumerator.MoveNext())
                    continue;
                
                MakeLast();
                break;
            }
            
            ExcludeProperty();
        }

        // Write fragments
        foreach (var frag in frags)
        {
            writer.Write(frag.Pack());
        }

        // Serialize zero mask
        if (zeroMask.Any(x => x))
        {
            var result = new byte[(zeroMask.Count - 1) / 8 + 1];
            var index = 0;

            for (int i = 0; i < zeroMask.Count; i++)
            {
                result[index] += Convert.ToByte((zeroMask[i] ? 1 : 0) * Math.Pow(2, i));

                if (i > 0 && i % 8 == 0)
                    index++;
            }
            
            writer.WriteBytes(result);
        }

        return;

        void AddFrag() => frags.Add(new FFragment());

        void TrimZeroMask(FFragment frag)
        {
            if (!frag.bHasAnyZeroes)
            {
                zeroMask.RemoveRange(zeroMask.Count - frag.ValueNum, frag.ValueNum);
            }
        }
	    
        void IncludeProperty(bool isZero)
        {
            if (GetLast().ValueNum == FFragment.ValueMax)
            {
                TrimZeroMask(GetLast());
                AddFrag();
            }
		    
            zeroMask.Add(isZero);
            frags[^1] = frags[^1] with
            {
                ValueNum = (byte)(frags[^1].ValueNum + 1),
                bHasAnyZeroes = frags[^1].bHasAnyZeroes | isZero
            };
        }

        void ExcludeProperty()
        {
            if (GetLast().ValueNum != 0 || GetLast().SkipNum == FFragment.SkipMax)
            {
                TrimZeroMask(GetLast());
                AddFrag();
            }

            frags[^1] = frags[^1] with
            {
                SkipNum = (byte)(frags[^1].SkipNum + 1)
            };
        }

        void MakeLast() => frags[^1] = frags[^1] with
        {
            bIsLast = true
        };

        FFragment GetLast() => frags[^1];
    }
}
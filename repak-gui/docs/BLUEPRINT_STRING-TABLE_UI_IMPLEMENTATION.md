# Blueprint & Text (StringTable) Additive Categories - UI Implementation Guide

## Overview
This document outlines the UI changes needed to support **additive categories** for Blueprint and Text mods. These categories can now appear alongside primary categories (Mesh, VFX, etc.) rather than replacing them.

## Backend Status
✅ **Complete** - Additive category system implemented:
- Blueprint and Text are now **additive categories** that don't override the main category
- Returns `additional_categories: Vec<String>` in `ModCharacteristics` struct
- The `mod_type` string now includes additional categories: `"Blade - Mesh [Blueprint, Text]"`
- Blueprint detection patterns:
  - `BP_Something` (Blueprint prefix)
  - `Something_C` (Blueprint class suffix)
  - `SomethingBP` (Blueprint suffix)
  - Files in `/Blueprints/` folder
- Text detection patterns:
  - Files in `/StringTable/` folder
  - Files in `/Data/StringTable/` folder

## Frontend Implementation Needed

### 1. Update Type Filters (Left Sidebar)

**Location**: Where mod type filters are generated (likely in main App component)

**Current Behavior**:
- Filters are generated from the `category` field only
- Shows: Audio, Mesh, UI, VFX, etc.

**Required Change**:
- **Also** extract categories from `additional_categories` array
- Add "Blueprint" and "Text" as filter options when mods with these categories exist

**Implementation**:
```javascript
// When building the list of available filter types:
const allCategories = new Set();

mods.forEach(mod => {
  // Add main category
  if (mod.category) {
    allCategories.add(mod.category);
  }
  
  // Add additional categories (Blueprint, Text)
  if (mod.additional_categories) {
    mod.additional_categories.forEach(cat => {
      allCategories.add(cat);
    });
  }
});

// Convert to array for rendering
const filterTypes = Array.from(allCategories).sort();
```

**Filtering Logic**:
```javascript
// When filtering mods by selected type:
const filteredMods = mods.filter(mod => {
  if (!selectedType) return true;
  
  // Check if main category matches
  if (mod.category === selectedType) return true;
  
  // Check if any additional category matches
  if (mod.additional_categories?.includes(selectedType)) return true;
  
  return false;
});
```

### 2. ModDetailsPanel.jsx Changes

**Location**: `src/components/ModDetailsPanel.jsx`

**Current State**:
```jsx
<div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
  {details.character_name && (
    <div className="character-badge" title="Character">
      {details.character_name}
    </div>
  )}
  <div className="category-badge" title="Mod Type">
    {details.category || 'Unknown'}
  </div>
  {details.is_iostore && (
    <div className="iostore-badge">IoStore Package</div>
  )}
</div>
```

**Proposed Change**:
```jsx
<div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
  {details.character_name && (
    <div className="character-badge" title="Character">
      {details.character_name}
    </div>
  )}
  <div className="category-badge" title="Mod Type">
    {details.category || 'Unknown'}
  </div>
  {/* Render additional categories (Blueprint, Text) */}
  {details.additional_categories?.map(cat => (
    <div 
      key={cat}
      className={`additional-badge ${cat.toLowerCase()}-badge`}
      title={`Contains ${cat}`}
    >
      {cat}
    </div>
  ))}
  {details.is_iostore && (
    <div className="iostore-badge">IoStore Package</div>
  )}
</div>
```

**Key Points**:
- Dynamically render badges for each additional category
- Uses class names like `blueprint-badge` and `text-badge` for styling
- Appears between main category and IoStore badge

### 3. App.css Changes

**Location**: `src/App.css`

**Current State**:
```css
/* Character/Category badges */
.character-badge,
.category-badge {
  display: inline-block;
  padding: 0.4rem 0.75rem;
  border-radius: 999px;
  font-weight: 600;
  font-size: 0.85rem;
}

.character-badge {
  background: rgba(255,255,255,0.08);
  color: var(--text-primary);
  border: 1px solid var(--panel-border);
}

.category-badge {
  background: #4a9eff20;
  color: var(--accent-primary, #4a9eff);
  border: 1px solid rgba(74,158,255,0.35);
}
```

**Proposed Change**:
```css
/* Character/Category badges */
.character-badge,
.category-badge,
.additional-badge {
  display: inline-block;
  padding: 0.4rem 0.75rem;
  border-radius: 999px;
  font-weight: 600;
  font-size: 0.85rem;
}

.character-badge {
  background: rgba(255,255,255,0.08);
  color: var(--text-primary);
  border: 1px solid var(--panel-border);
}

.category-badge {
  background: #4a9eff20;
  color: var(--accent-primary, #4a9eff);
  border: 1px solid rgba(74,158,255,0.35);
}

/* Additional category badges */
.blueprint-badge {
  background: #9b59b620;
  color: #c084fc;
  border: 1px solid rgba(192,132,252,0.35);
}

.text-badge {
  background: #fbbf2420;
  color: #fbbf24;
  border: 1px solid rgba(251,191,36,0.35);
}
```

**Design Notes**:
- Blueprint: Purple color scheme (`#c084fc`) 
- Text: Yellow/Gold color scheme (`#fbbf24`)
- Both match existing badge styling (rounded, semi-transparent background)
- Feel free to adjust colors to match your design system

## Visual Layout

**Badge Order** (left to right):
```
[Character Name] [Category] [Blueprint] [Text] [IoStore Package]
```

**Example Displays**:
```
Invisible Woman    Mesh    Blueprint    Text    IoStore Package
```
```
Blade    VFX    Text
```
```
Hawkeye    Audio    IoStore Package
```
```
Unknown    Blueprint
```

## Alternative Design Considerations

### Option 1: Icon-based Badges
Instead of text, use icons:
```jsx
{details.additional_categories?.map(cat => (
  <div key={cat} className={`${cat.toLowerCase()}-badge`}>
    {cat === 'Blueprint' ? '📘' : '📝'} {cat}
  </div>
))}
```

### Option 2: Smaller Indicators
Make additional badges smaller/more subtle:
```css
.additional-badge {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  /* ... */
}
```

### Option 3: Abbreviated Text
Use shorter labels:
```jsx
{cat === 'Blueprint' ? 'BP' : 'TXT'}
```

## Testing Checklist

### Filter Functionality:
- [ ] "Blueprint" appears in type filters when Blueprint mods exist
- [ ] "Text" appears in type filters when Text mods exist
- [ ] Clicking "Blueprint" filter shows all mods with Blueprint (including Mesh+Blueprint)
- [ ] Clicking "Text" filter shows all mods with Text (including VFX+Text)
- [ ] Filters work correctly for mods with multiple additional categories

### Badge Display:
- [ ] Blueprint badge appears for mods with Blueprint files
- [ ] Text badge appears for mods with StringTable files
- [ ] Both badges can appear together on the same mod
- [ ] Badges do NOT appear for mods without those categories
- [ ] Badge styling matches existing design system
- [ ] Badges are responsive (wrap properly on small screens)
- [ ] Tooltips show correct information on hover
- [ ] Badge order is correct (Character → Category → Additional → IoStore)

## Data Available from Backend

The `ModCharacteristics` object (part of mod data) now includes:
```typescript
interface ModCharacteristics {
  mod_type: string;              // e.g., "Blade - Mesh [Blueprint, Text]"
  heroes: string[];              // e.g., ["Blade"]
  character_name: string;        // e.g., "Blade"
  category: string;              // e.g., "Mesh" (primary category)
  additional_categories: string[]; // ← NEW: e.g., ["Blueprint", "Text"]
}
```

**Important Notes**:
- `mod_type` string now includes additional categories in square brackets for display
- `category` contains only the primary category (Mesh, VFX, Audio, etc.)
- `additional_categories` is an array that can contain "Blueprint", "Text", or both
- For filtering, you need to check BOTH `category` AND `additional_categories`

## Questions for UX Designer

1. **Colors**: 
   - Blueprint: Purple (#c084fc) - OK?
   - Text: Yellow/Gold (#fbbf24) - OK?
   - Or should they use different colors?

2. **Display Format**:
   - Full text "Blueprint" and "Text"?
   - Abbreviated "BP" and "TXT"?
   - Icons (📘 for Blueprint, 📝 for Text)?

3. **Badge Size**:
   - Same size as category badge?
   - Smaller/more subtle?

4. **Filter Display**:
   - Should Blueprint/Text filters be visually distinct from primary category filters?
   - Should they be in a separate section?

5. **Interaction States**:
   - Any specific hover/active states needed?
   - Should clicking a badge in mod details apply that filter?

## Implementation Priority

**High Priority** (Required for functionality):
1. Update filter generation to include `additional_categories`
2. Update filter logic to check both `category` and `additional_categories`
3. Add basic badge rendering for additional categories
4. Add CSS for `.blueprint-badge` and `.text-badge`

**Medium Priority** (Polish):
- Custom icon/styling refinements
- Hover states/animations
- Visual distinction in filter sidebar

**Low Priority** (Nice to have):
- Alternative display options (icons, abbreviations)
- Click-to-filter from badges
- Special visual treatment in mod list view

## Example Mod Data

**Mesh mod with Blueprint and Text:**
```json
{
  "mod_type": "Invisible Woman - Mesh [Blueprint, Text]",
  "character_name": "Invisible Woman",
  "category": "Mesh",
  "additional_categories": ["Blueprint", "Text"]
}
```

**VFX mod with Text only:**
```json
{
  "mod_type": "Blade - VFX [Text]",
  "character_name": "Blade",
  "category": "VFX",
  "additional_categories": ["Text"]
}
```

**Blueprint-only mod:**
```json
{
  "mod_type": "Hawkeye - Blueprint",
  "character_name": "Hawkeye",
  "category": "Blueprint",
  "additional_categories": []
}
```

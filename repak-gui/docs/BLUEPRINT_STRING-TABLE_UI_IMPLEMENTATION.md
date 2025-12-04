# Blueprint Badge UI Implementation Proposal

## Overview
This document outlines the proposed UI changes to display Blueprint detection as a separate badge in the mod details panel.

## Backend Status
✅ **Complete** - Blueprint detection is now automatic and efficient:
- Detects Blueprints using filename patterns (instant, no file extraction)
- Returns `has_blueprint: bool` in `ModDetails` struct
- Patterns detected:
  - `BP_Something` (Blueprint prefix)
  - `Something_C` (Blueprint class suffix)
  - `SomethingBP` (Blueprint suffix)
  - Files in `/Blueprints/` folder

## Frontend Implementation Needed

### 1. ModDetailsPanel.jsx Changes

**Location**: `src/components/ModDetailsPanel.jsx`

**Current State** (lines 54-66):
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
  {details.has_blueprint && (
    <div className="blueprint-badge" title="Contains Blueprints">
      Blueprint
    </div>
  )}
  {details.is_iostore && (
    <div className="iostore-badge">IoStore Package</div>
  )}
</div>
```

**Key Points**:
- Add Blueprint badge between category and IoStore badges
- Conditional rendering based on `details.has_blueprint`
- Uses new `blueprint-badge` CSS class

### 2. App.css Changes

**Location**: `src/App.css`

**Current State** (lines 1104-1123):
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
.blueprint-badge {
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

.blueprint-badge {
  background: #9b59b620;
  color: #c084fc;
  border: 1px solid rgba(192,132,252,0.35);
}
```

**Design Notes**:
- Purple color scheme (`#c084fc`) to distinguish from other badges
- Matches existing badge styling (rounded, semi-transparent background)
- Feel free to adjust colors to match your design system

## Visual Layout

**Badge Order** (left to right):
```
[Character Name] [Category] [Blueprint] [IoStore Package]
```

**Example Displays**:
```
Luna Snow    Mesh    Blueprint    IoStore Package
```
```
Hawkeye    Audio    IoStore Package
```
```
Unknown    Text    Blueprint
```

## Alternative Design Considerations

### Option 1: Icon-based Badge
Instead of text "Blueprint", could use an icon:
```jsx
{details.has_blueprint && (
  <div className="blueprint-badge" title="Contains Blueprints">
    📘 BP
  </div>
)}
```

### Option 2: Smaller Indicator
Make Blueprint badge smaller/more subtle:
```css
.blueprint-badge {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  /* ... */
}
```

### Option 3: Combined with Category
Show Blueprint as part of category badge:
```jsx
<div className="category-badge" title="Mod Type">
  {details.category || 'Unknown'}
  {details.has_blueprint && ' + Blueprint'}
</div>
```

## Testing Checklist

- [ ] Blueprint badge appears for mods with Blueprint files
- [ ] Blueprint badge does NOT appear for non-Blueprint mods
- [ ] Badge styling matches existing design system
- [ ] Badge is responsive (wraps properly on small screens)
- [ ] Tooltip shows "Contains Blueprints" on hover
- [ ] Badge order is correct (Character → Category → Blueprint → IoStore)

## Data Available from Backend

The `ModDetails` object returned from `get_mod_details` includes:
```typescript
interface ModDetails {
  mod_name: string;
  mod_type: string;
  character_name: string;
  category: string;
  file_count: number;
  total_size: number;
  files: string[];
  is_iostore: boolean;
  has_blueprint: boolean;  // ← NEW
}
```

## Questions for UX Designer

1. Should Blueprint badge use purple (#c084fc) or a different color?
2. Should it display text "Blueprint" or an icon/abbreviation?
3. Should it be the same size as other badges or smaller?
4. Any specific hover/interaction states needed?
5. Should Blueprint mods have any special visual treatment in the mod list?

## Implementation Priority

**High Priority**:
- Basic Blueprint badge display (text-based, purple)
- Conditional rendering based on `has_blueprint`

**Medium Priority**:
- Custom icon/styling refinements
- Hover states/animations

**Low Priority**:
- Alternative display options
- Integration with mod list view

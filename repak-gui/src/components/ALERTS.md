# Alert System Documentation

A toast notification system for Repak X with card stacking, animations, and various alert types.

## Quick Start

```jsx
import { useAlert } from './components/AlertHandler';

function MyComponent() {
  const alert = useAlert();
  
  // Show a success alert
  alert.success('Done!', 'Operation completed successfully');
}
```

---

## Setup

The `AlertProvider` must wrap your app (already done in `App.jsx`):

```jsx
import AlertProvider from './components/AlertHandler';

function App() {
  return (
    <AlertProvider placement="bottom-center">
      <YourApp />
    </AlertProvider>
  );
}
```

### Placement Options
- `bottom-center` (default)
- `bottom-right`
- `bottom-left`
- `top-center`
- `top-right`
- `top-left`

---

## Basic Usage

### Using the Hook

```jsx
const alert = useAlert();
```

### Convenience Methods

```jsx
// Success (green)
alert.success('Title', 'Description');

// Error (red)
alert.error('Title', 'Description');

// Warning (orange)
alert.warning('Title', 'Description');

// Info (blue)
alert.info('Title', 'Description');
```

### Full Configuration

```jsx
alert.showAlert({
  title: 'Alert Title',
  description: 'Alert description text',
  color: 'success',      // 'success' | 'danger' | 'warning' | 'primary' | 'secondary' | 'default'
  variant: 'flat',       // 'flat' | 'solid' | 'bordered' | 'faded'
  duration: 5000,        // Auto-dismiss in ms (0 = no auto-dismiss)
  hideIcon: false,       // Hide the icon
  icon: <CustomIcon />,  // Custom icon component
});
```

---

## Promise Toast

Shows a loading state while waiting for an async operation, then updates to success/error.

```jsx
// Basic usage
alert.promise(
  fetch('/api/save'),
  {
    loading: { title: 'Saving...', description: 'Please wait' },
    success: { title: 'Saved!', description: 'Changes saved successfully' },
    error: { title: 'Failed', description: 'Could not save changes' }
  }
);

// With dynamic messages based on result
alert.promise(
  fetchData(),
  {
    loading: { title: 'Loading...' },
    success: (data) => ({ 
      title: 'Loaded!', 
      description: `Got ${data.length} items` 
    }),
    error: (err) => ({ 
      title: 'Error', 
      description: err.message 
    })
  }
);

// With a function that returns a promise
alert.promise(
  () => someAsyncFunction(param1, param2),
  { loading, success, error }
);
```

---

## endContent (Action Buttons)

Add interactive buttons to alerts:

```jsx
alert.showAlert({
  title: 'Update Available',
  description: 'Version 2.0 is ready',
  color: 'primary',
  endContent: (
    <button 
      className="toast-action-btn"
      onClick={() => handleUpdate()}
    >
      Update Now
    </button>
  )
});
```

The `toast-action-btn` class provides hover effects. You can also use custom styling.

---

## Programmatic Control

### Dismiss a Specific Alert

```jsx
const id = alert.success('Title', 'Description');

// Later...
alert.dismissAlert(id);
```

### Dismiss All Alerts

```jsx
alert.dismissAllAlerts();
```

### Update an Existing Alert

```jsx
const id = alert.showAlert({ title: 'Initial', color: 'default' });

// Later...
alert.updateToast(id, { 
  title: 'Updated!', 
  color: 'success' 
});
```

---

## Available Colors

| Color       | Use Case                              |
| ----------- | ------------------------------------- |
| `success`   | Completed operations, confirmations   |
| `danger`    | Errors, failures, destructive actions |
| `warning`   | Cautions, potential issues            |
| `primary`   | Info, updates, general notifications  |
| `secondary` | Secondary info, less important        |
| `default`   | Neutral messages                      |

---

## Available Variants

| Variant    | Style                       |
| ---------- | --------------------------- |
| `flat`     | Subtle background (default) |
| `solid`    | Full color background       |
| `bordered` | Outline style               |
| `faded`    | Muted/faded appearance      |

---

## Behavior

- **Card Stacking**: Up to 3 alerts visible, stacked like cards
- **Hover to Expand**: Hovering fans out stacked cards
- **Progress Bar**: Shows on the front alert when auto-dismissing
- **Pause on Hover**: Auto-dismiss pauses when hovering
- **Clear All**: Button appears when 2+ alerts are active

---

## Examples

### Installation Success
```jsx
alert.success(
  'Mod Installed', 
  'Luna Snow - Ice Empress has been added'
);
```

### Error with Details
```jsx
alert.error(
  'Installation Failed',
  'The file appears to be corrupted or password-protected'
);
```

### Async Operation
```jsx
alert.promise(
  invoke('install_mod', { path: modPath }),
  {
    loading: { title: 'Installing...', description: modName },
    success: { title: 'Installed!', description: `${modName} is ready` },
    error: (e) => ({ title: 'Failed', description: e.message })
  }
);
```

### With Action Button
```jsx
alert.showAlert({
  title: 'Backup Reminder',
  description: 'You have 50 mods not backed up',
  color: 'warning',
  endContent: (
    <button className="toast-action-btn" onClick={handleBackup}>
      Backup Now
    </button>
  )
});
```

---

## API Reference

### `useAlert()` Hook Returns

| Method                        | Description                             |
| ----------------------------- | --------------------------------------- |
| `showAlert(config)`           | Show alert with full config, returns ID |
| `success(title, desc, opts?)` | Show success alert                      |
| `error(title, desc, opts?)`   | Show error alert                        |
| `warning(title, desc, opts?)` | Show warning alert                      |
| `info(title, desc, opts?)`    | Show info alert                         |
| `promise(promise, config)`    | Show loading → success/error            |
| `dismissAlert(id)`            | Dismiss specific alert                  |
| `dismissAllAlerts()`          | Dismiss all alerts                      |
| `updateToast(id, updates)`    | Update existing alert                   |

### Alert Config Object

| Property      | Type      | Default     | Description        |
| ------------- | --------- | ----------- | ------------------ |
| `title`       | string    | -           | Alert title        |
| `description` | string    | -           | Alert description  |
| `color`       | string    | `'default'` | Color variant      |
| `variant`     | string    | `'flat'`    | Style variant      |
| `duration`    | number    | `5000`      | Auto-dismiss (ms)  |
| `icon`        | ReactNode | auto        | Custom icon        |
| `hideIcon`    | boolean   | `false`     | Hide icon          |
| `endContent`  | ReactNode | -           | Right-side content |
| `isLoading`   | boolean   | `false`     | Show spinner       |

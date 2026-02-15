import React from 'react'
import './Checkbox.css'

/**
 * repakx-style Checkbox component
 *
 * @param {Object} props
 * @param {boolean} [props.checked] - Controlled checked state
 * @param {boolean} [props.defaultChecked=false] - Initial state for uncontrolled usage
 * @param {'sm'|'md'|'lg'} [props.size='md'] - Visual size of the control
 * @param {'default'|'primary'|'secondary'|'success'|'warning'|'danger'} [props.color='primary'] - Accent color
 * @param {boolean} [props.isDisabled=false] - Whether the checkbox is disabled
 * @param {boolean} [props.isIndeterminate=false] - Whether the checkbox is in an indeterminate state
 * @param {'circle'|'rounded'} [props.radius='rounded'] - Shape of the checkbox
 * @param {function} [props.onChange] - Callback receiving (checked:boolean, event:MouseEvent)
 * @param {React.ReactNode} [props.children] - Optional label content rendered to the right
 * @param {string} [props.className] - Optional extra class names
 */
const Checkbox = ({
  checked,
  defaultChecked = false,
  size = 'md',
  color = 'primary',
  isDisabled = false,
  isIndeterminate = false,
  radius = 'rounded',
  onChange,
  children,
  className = '',
  ...props
}) => {
  const isControlled = typeof checked === 'boolean'
  const [internalChecked, setInternalChecked] = React.useState(defaultChecked)
  const currentChecked = isControlled ? checked : internalChecked

  React.useEffect(() => {
    if (isControlled) return
    setInternalChecked(defaultChecked)
  }, [defaultChecked, isControlled])

  const handleToggle = (event) => {
    if (isDisabled) return
    const nextValue = !currentChecked
    if (!isControlled) {
      setInternalChecked(nextValue)
    }
    if (onChange) {
      onChange(nextValue, event)
    }
  }

  const classes = [
    'repakx-checkbox',
    size,
    color,
    radius,
    currentChecked ? 'checked' : '',
    isDisabled ? 'disabled' : '',
    isIndeterminate ? 'indeterminate' : '',
    className
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <label className={classes} data-checked={currentChecked} data-disabled={isDisabled}>
      <button
        type="button"
        className="repakx-checkbox-control"
        role="checkbox"
        aria-checked={isIndeterminate ? 'mixed' : currentChecked}
        aria-disabled={isDisabled}
        onClick={handleToggle}
        disabled={isDisabled}
        {...props}
      >
        <span className="repakx-checkbox-box">
          {isIndeterminate ? (
            <svg className="repakx-checkbox-icon" viewBox="0 0 24 24" fill="none">
              <line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
            </svg>
          ) : (
            <svg className="repakx-checkbox-icon" viewBox="0 0 24 24" fill="none">
              <polyline points="4 12 9 17 20 6" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          )}
        </span>
      </button>
      {children && <span className="repakx-checkbox-label">{children}</span>}
    </label>
  )
}

export { Checkbox }
export default Checkbox

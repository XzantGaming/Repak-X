import { useState, useEffect, useRef } from 'react'
import PropTypes from 'prop-types'
import './NumberInput.css'

const NumberInput = ({ value, min = 0, max = 999, onChange, className, disabled }) => {
  const [localValue, setLocalValue] = useState(value)
  const [isOnCooldown, setIsOnCooldown] = useState(false)
  const inputRef = useRef(null)
  const cooldownTimerRef = useRef(null)

  useEffect(() => {
    setLocalValue(value)
  }, [value])

  useEffect(() => {
    return () => {
      if (cooldownTimerRef.current) {
        clearTimeout(cooldownTimerRef.current)
      }
    }
  }, [])

  const commitValue = (newValue) => {
    if (disabled) return
    // If empty string or NaN, revert to original value
    if (newValue === '' || isNaN(newValue)) {
      setLocalValue(value)
      return
    }

    let clamped = Math.max(min, Math.min(max, newValue))

    setLocalValue(clamped)
    if (onChange && clamped !== value) {
      onChange(clamped)
    }
  }

  const handleBlur = () => {
    commitValue(localValue)
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') {
      inputRef.current?.blur()
    }
  }

  const triggerCooldown = () => {
    setIsOnCooldown(true)
    if (cooldownTimerRef.current) {
      clearTimeout(cooldownTimerRef.current)
    }
    cooldownTimerRef.current = setTimeout(() => {
      setIsOnCooldown(false)
    }, 2000)
  }

  const handleIncrement = (e) => {
    e.stopPropagation()
    if (disabled || isOnCooldown) return

    const newValue = Math.min(max, localValue + 1)
    setLocalValue(newValue)
    if (onChange && newValue !== value) {
      onChange(newValue)
      triggerCooldown()
    }
  }

  const handleDecrement = (e) => {
    e.stopPropagation()
    if (disabled || isOnCooldown) return

    const newValue = Math.max(min, localValue - 1)
    setLocalValue(newValue)
    if (onChange && newValue !== value) {
      onChange(newValue)
      triggerCooldown()
    }
  }

  const handleChange = (e) => {
    if (disabled) return
    const inputValue = e.target.value
    if (inputValue === '') {
      setLocalValue('')
      return
    }
    const val = parseInt(inputValue, 10)
    setLocalValue(isNaN(val) ? '' : val)
  }

  return (
    <div className={`number-input-container ${className || ''} ${disabled ? 'disabled' : ''}`} onClick={(e) => e.stopPropagation()}>
      <button
        type="button"
        className="number-btn minus"
        onClick={handleDecrement}
        disabled={disabled || localValue <= min || isOnCooldown}
      >
        −
      </button>
      <input
        ref={inputRef}
        type="number"
        className="number-display"
        value={localValue}
        onChange={handleChange}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        min={min}
        max={max}
        disabled={disabled}
      />
      <button
        type="button"
        className="number-btn plus"
        onClick={handleIncrement}
        disabled={disabled || localValue >= max || isOnCooldown}
      >
        +
      </button>
    </div>
  )
}

NumberInput.propTypes = {
  value: PropTypes.number,
  min: PropTypes.number,
  max: PropTypes.number,
  onChange: PropTypes.func,
  className: PropTypes.string,
  disabled: PropTypes.bool
}

export default NumberInput

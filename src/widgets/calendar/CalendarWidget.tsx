import {
  eachDayOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  isSameDay,
  isSameMonth,
  startOfMonth,
  startOfWeek,
} from 'date-fns'
import { useEffect, useState } from 'react'
import styles from './CalendarWidget.module.css'

const DAY_HEADERS = ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su']

export function CalendarWidget() {
  const [now, setNow] = useState(() => new Date())

  useEffect(() => {
    // Align tick to the next whole minute
    const msUntilNextMinute = 60_000 - (Date.now() % 60_000)
    let intervalId: ReturnType<typeof setInterval>
    const timeoutId = setTimeout(() => {
      setNow(new Date())
      intervalId = setInterval(() => setNow(new Date()), 60_000)
    }, msUntilNextMinute)
    return () => {
      clearTimeout(timeoutId)
      clearInterval(intervalId)
    }
  }, [])

  const monthStart = startOfMonth(now)
  const monthEnd = endOfMonth(now)
  const calStart = startOfWeek(monthStart, { weekStartsOn: 1 })
  const calEnd = endOfWeek(monthEnd, { weekStartsOn: 1 })
  const days = eachDayOfInterval({ start: calStart, end: calEnd })

  return (
    <div className={styles.widget}>
      <div className={styles.header}>
        <span className={styles.month}>{format(now, 'MMMM yyyy')}</span>
        <span className={styles.time}>{format(now, 'HH:mm')}</span>
      </div>
      <div className={styles.grid}>
        {DAY_HEADERS.map((d) => (
          <span key={d} className={styles.dayHeader}>
            {d}
          </span>
        ))}
        {days.map((day) => {
          const cls = [
            styles.day,
            !isSameMonth(day, now) ? styles.outside : '',
            isSameDay(day, now) ? styles.today : '',
          ]
            .filter(Boolean)
            .join(' ')
          return (
            <span key={day.toISOString()} className={cls}>
              {format(day, 'd')}
            </span>
          )
        })}
      </div>
    </div>
  )
}

import React from 'react'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card.tsx'

interface EarningsData {
  total_earnings: number
  today_earnings: number
  yesterday_earnings: number
  this_week_earnings: number
  this_month_earnings: number
  daily_earnings: Record<string, number>
  monthly_earnings: Record<string, number>
  pending_rewards: number
  last_payout?: string
  next_payout?: string
}

export default function EarningsCard() {
  const { data: earnings, isLoading } = useQuery({
    queryKey: ['earnings'],
    queryFn: () => invoke<EarningsData>('get_earnings'),
    refetchInterval: 30000, // Refresh every 30 seconds
  })

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Earnings</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="animate-pulse">
            <div className="h-8 bg-gray-200 rounded mb-4"></div>
            <div className="space-y-2">
              <div className="h-4 bg-gray-200 rounded"></div>
              <div className="h-4 bg-gray-200 rounded"></div>
              <div className="h-4 bg-gray-200 rounded"></div>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  const formatCurrency = (cents: number) => `$${(cents / 100).toFixed(2)}`
  const todayChange = earnings?.today_earnings && earnings?.yesterday_earnings 
    ? ((earnings.today_earnings - earnings.yesterday_earnings) / earnings.yesterday_earnings * 100)
    : 0

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span>Earnings</span>
        </CardTitle>
        <CardDescription>Your provider node earnings</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Total Earnings */}
        <div className="text-center p-4 bg-card rounded-lg border border-primary/20">
          <div className="text-3xl font-bold text-primary">
            {formatCurrency(earnings?.total_earnings || 0)}
          </div>
          <div className="text-sm text-muted-foreground mt-1">Total Earned</div>
        </div>

        {/* Today's Earnings */}
        <div className="flex items-center justify-between p-3 bg-card rounded-lg">
          <div>
            <div className="font-semibold text-primary">{formatCurrency(earnings?.today_earnings || 0)}</div>
            <div className="text-sm text-muted-foreground">Today</div>
          </div>
          <div className={`text-sm font-medium ${
            todayChange >= 0 ? 'text-primary' : 'text-destructive'
          }`}>
            {todayChange >= 0 ? '↗' : '↘'} {Math.abs(todayChange).toFixed(1)}%
          </div>
        </div>

        {/* Weekly Earnings */}
        <div className="flex items-center justify-between p-3 bg-card rounded-lg">
          <div>
            <div className="font-semibold text-primary">{formatCurrency(earnings?.this_week_earnings || 0)}</div>
            <div className="text-sm text-muted-foreground">This Week</div>
          </div>
          <div className="text-sm text-muted-foreground">
            {formatCurrency((earnings?.this_week_earnings || 0) / 7)} avg/day
          </div>
        </div>

        {/* Monthly Earnings */}
        <div className="flex items-center justify-between p-3 bg-card rounded-lg">
          <div>
            <div className="font-semibold text-primary">{formatCurrency(earnings?.this_month_earnings || 0)}</div>
            <div className="text-sm text-muted-foreground">This Month</div>
          </div>
          <div className="text-sm text-muted-foreground">
            {formatCurrency((earnings?.this_month_earnings || 0) / 30)} avg/day
          </div>
        </div>

        {/* Pending Rewards */}
        {(earnings?.pending_rewards || 0) > 0 && (
          <div className="p-3 bg-card border border-primary/20 rounded-lg">
            <div className="flex items-center justify-between">
              <div>
                <div className="font-semibold text-primary">
                  {formatCurrency(earnings?.pending_rewards || 0)}
                </div>
                <div className="text-sm text-muted-foreground">Pending Payout</div>
              </div>
              <div className="text-primary">Pending</div>
            </div>
            {earnings?.next_payout && (
              <div className="text-xs text-muted-foreground mt-2">
                Next payout: {new Date(earnings.next_payout).toLocaleDateString()}
              </div>
            )}
          </div>
        )}

        {/* Last Week Performance */}
        <div className="pt-4 border-t">
          <div className="text-sm font-medium text-gray-700 mb-2">Last 7 Days</div>
          <div className="flex justify-between items-end h-12 space-x-1">
            {Array.from({ length: 7 }, (_, i) => {
              const date = new Date()
              date.setDate(date.getDate() - (6 - i))
              const dateStr = date.toISOString().split('T')[0]
              const earning = earnings?.daily_earnings?.[dateStr] || 0
              const maxEarning = Math.max(...Object.values(earnings?.daily_earnings || {}))
              const height = maxEarning > 0 ? (earning / maxEarning) * 100 : 0
              
              return (
                <div key={i} className="flex flex-col items-center flex-1">
                  <div 
                    className="w-full bg-blue-200 rounded-t min-h-[2px] transition-all duration-300"
                    style={{ height: `${Math.max(height, 8)}%` }}
                    title={`${date.toLocaleDateString()}: ${formatCurrency(earning)}`}
                  />
                  <div className="text-xs text-gray-500 mt-1">
                    {date.toLocaleDateString('en', { weekday: 'narrow' })}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
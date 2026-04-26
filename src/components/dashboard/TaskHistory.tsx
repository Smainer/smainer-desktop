import React, { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'

interface TaskHistoryEntry {
  task_id: string
  task_type: string
  status: string
  submitted_at: string
  started_at?: string
  completed_at?: string
  duration?: number
  reward?: number
  client_id: string
  gpu_used: boolean
  error_message?: string
}

const taskTypeLabels = {
  image_generation: 'Image',
  text_processing: 'Text', 
  model_training: 'Training',
  data_analysis: 'Analysis',
}

const statusColors = {
  pending: 'text-muted-foreground bg-card border-border',
  running: 'text-primary bg-card border-primary/20',
  completed: 'text-primary bg-card border-primary/20',
  failed: 'text-destructive bg-card border-destructive/20',
}

export default function TaskHistory() {
  const [limit, setLimit] = useState(50)
  const [filter, setFilter] = useState<string>('all')

  const { data: tasks, isLoading, error } = useQuery({
    queryKey: ['taskHistory', limit],
    queryFn: () => invoke<TaskHistoryEntry[]>('get_task_history', { limit }),
    refetchInterval: 10000, // Refresh every 10 seconds
  })

  const filteredTasks = tasks?.filter(task => 
    filter === 'all' || task.status === filter
  ) || []

  const formatCurrency = (cents: number) => `$${(cents / 100).toFixed(2)}`
  const formatDuration = (seconds: number) => {
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60
    return `${minutes}m ${remainingSeconds}s`
  }

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Task History</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="animate-pulse space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-16 bg-card rounded"></div>
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Task History</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center text-destructive">
            Failed to load task history. Please try again.
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Task History</CardTitle>
          <div className="flex items-center space-x-4">
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className="px-3 py-1 border border-border rounded-lg text-sm focus:ring-2 focus:ring-ring"
            >
              <option value="all">All Tasks</option>
              <option value="running">Running</option>
              <option value="completed">Completed</option>
              <option value="failed">Failed</option>
              <option value="pending">Pending</option>
            </select>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setLimit(limit + 50)}
            >
              Load More
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {filteredTasks.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            {filter === 'all' ? 'No tasks yet' : `No ${filter} tasks`}
          </div>
        ) : (
          <div className="space-y-3">
            {filteredTasks.map((task) => (
              <div
                key={task.task_id}
                className="border rounded-lg p-4 hover:bg-card transition-colors"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="text-sm font-medium text-primary px-2 py-1 bg-card rounded">
                      {taskTypeLabels[task.task_type as keyof typeof taskTypeLabels] || 'Task'}
                    </span>
                    <div>
                      <div className="font-medium text-sm">{task.task_id}</div>
                      <div className="text-xs text-muted-foreground capitalize">
                        {task.task_type.replace('_', ' ')}
                        {task.gpu_used && ' • GPU'}
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-4">
                    <span
                      className={`px-2 py-1 rounded-full text-xs font-medium border ${
                        statusColors[task.status as keyof typeof statusColors] || 
                        'text-muted-foreground bg-muted border-border'
                      }`}
                    >
                      {task.status}
                    </span>
                    
                    {task.reward && (
                      <div className="text-right">
                        <div className="font-semibold text-primary">
                          {formatCurrency(task.reward)}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
                
                <div className="mt-3 grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-muted-foreground">
                  <div>
                    <span className="font-medium">Submitted:</span>
                    <div>{new Date(task.submitted_at).toLocaleString()}</div>
                  </div>
                  
                  {task.started_at && (
                    <div>
                      <span className="font-medium">Started:</span>
                      <div>{new Date(task.started_at).toLocaleString()}</div>
                    </div>
                  )}
                  
                  {task.completed_at && (
                    <div>
                      <span className="font-medium">Completed:</span>
                      <div>{new Date(task.completed_at).toLocaleString()}</div>
                    </div>
                  )}
                  
                  {task.duration && (
                    <div>
                      <span className="font-medium">Duration:</span>
                      <div>{formatDuration(task.duration)}</div>
                    </div>
                  )}
                </div>
                
                {task.error_message && (
                  <div className="mt-2 p-2 bg-destructive/10 border border-destructive/20 rounded text-xs text-destructive">
                    <span className="font-medium">Error:</span> {task.error_message}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
        
        {/* Task Summary */}
        <div className="mt-6 pt-4 border-t">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
            <div className="bg-primary/10 p-3 rounded-lg">
              <div className="text-lg font-bold text-primary">
                {tasks?.filter(t => t.status === 'completed').length || 0}
              </div>
              <div className="text-xs text-muted-foreground">Completed</div>
            </div>
            <div className="bg-primary/5 p-3 rounded-lg">
              <div className="text-lg font-bold text-primary">
                {tasks?.filter(t => t.status === 'running').length || 0}
              </div>
              <div className="text-xs text-muted-foreground">Running</div>
            </div>
            <div className="bg-muted p-3 rounded-lg">
              <div className="text-lg font-bold text-muted-foreground">
                {tasks?.filter(t => t.status === 'pending').length || 0}
              </div>
              <div className="text-xs text-muted-foreground">Pending</div>
            </div>
            <div className="bg-destructive/10 p-3 rounded-lg">
              <div className="text-lg font-bold text-destructive">
                {tasks?.filter(t => t.status === 'failed').length || 0}
              </div>
              <div className="text-xs text-muted-foreground">Failed</div>
            </div>
          </div>
          
          <div className="mt-4 text-center text-sm text-muted-foreground">
            Total earnings from completed tasks: {' '}
            <span className="font-medium text-primary">
              {formatCurrency(
                tasks?.filter(t => t.status === 'completed' && t.reward)
                     .reduce((sum, t) => sum + (t.reward || 0), 0) || 0
              )}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
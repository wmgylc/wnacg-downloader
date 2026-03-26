import { computed, defineComponent, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  NButton,
  NCard,
  NEmpty,
  NImage,
  NInput,
  NProgress,
  NSpace,
  NTag,
  useMessage,
} from 'naive-ui'

type DownloadTask = {
  id: string
  target: string
  status: 'downloading' | 'success' | 'failed'
  title?: string
  cover?: string
  totalPages?: number
  completedPages: number
  error?: string
  zipPath?: string
  createdAt: string
  updatedAt: string
  finishedAt?: string
}

const DEFAULT_API_BASE =
  typeof window !== 'undefined' ? `${window.location.origin}/api` : 'http://10.10.10.206:3000'

export default defineComponent({
  name: 'WebDownloadDashboard',
  setup() {
    const message = useMessage()
    const apiBase = ref(DEFAULT_API_BASE)
    const url = ref('')
    const tasks = ref<DownloadTask[]>([])
    const submitting = ref(false)
    let timer: number | undefined

    const sortedTasks = computed(() =>
      [...tasks.value].sort((a, b) => Number(b.updatedAt) - Number(a.updatedAt)),
    )
    const groupedTasks = computed(() => ({
      downloading: sortedTasks.value.filter((task) => task.status === 'downloading'),
      success: sortedTasks.value.filter((task) => task.status === 'success'),
      failed: sortedTasks.value.filter((task) => task.status === 'failed'),
    }))

    async function loadTasks() {
      try {
        const response = await fetch(`${apiBase.value}/tasks`)
        const payload = await response.json()
        tasks.value = payload.tasks ?? []
      } catch (error) {
        console.error(error)
      }
    }

    async function startDownload() {
      const target = url.value.trim()
      if (!target) {
        message.warning('请输入漫画详情页 URL')
        return
      }

      submitting.value = true
      try {
        const requestUrl = new URL(`download/start`, `${apiBase.value.replace(/\/+$/, '')}/`)
        requestUrl.searchParams.set('target', target)
        const response = await fetch(requestUrl)
        const payload = await response.json()
        if (!response.ok) {
          throw new Error(payload.error || payload.stderr || '创建下载任务失败')
        }
        url.value = ''
        message.success('下载任务已创建')
        await loadTasks()
      } catch (error) {
        message.error(error instanceof Error ? error.message : '创建下载任务失败')
      } finally {
        submitting.value = false
      }
    }

    function statusType(status: DownloadTask['status']) {
      switch (status) {
        case 'success':
          return 'success'
        case 'failed':
          return 'error'
        default:
          return 'warning'
      }
    }

    function statusLabel(status: DownloadTask['status']) {
      switch (status) {
        case 'success':
          return '下载完成'
        case 'failed':
          return '下载失败'
        default:
          return '下载中'
      }
    }

    function progressPercentage(task: DownloadTask) {
      if (!task.totalPages || task.totalPages <= 0) {
        return task.status === 'success' ? 100 : 0
      }
      return Math.min(100, Math.round((task.completedPages / task.totalPages) * 100))
    }

    onMounted(async () => {
      await loadTasks()
      timer = window.setInterval(loadTasks, 2000)
    })

    onBeforeUnmount(() => {
      if (timer) {
        window.clearInterval(timer)
      }
    })

    return () => (
      <div class="min-h-screen bg-[radial-gradient(circle_at_top,#fff7ed_0%,#fff 38%,#f8fafc_100%)] text-slate-900">
        <div class="mx-auto max-w-6xl px-4 py-8 md:px-8">
          <div class="mb-8 overflow-hidden rounded-6 border border-solid border-orange-200 bg-white/88 p-6 shadow-[0_24px_80px_rgba(15,23,42,0.08)] backdrop-blur">
            <div class="mb-2 text-sm font-600 uppercase tracking-[0.22em] text-orange-500">WNACG HTTP Downloader</div>
            <div class="mb-6 max-w-3xl text-4xl font-800 leading-tight">
              输入漫画详情页 URL，直接发起下载任务并在页面里追踪状态
            </div>
            <div class="mb-3 text-sm text-slate-500">当前 API: {apiBase.value}</div>
            <div class="flex flex-col gap-3 md:flex-row">
              <NInput
                value={url.value}
                onUpdate:value={(value) => (url.value = value)}
                placeholder="https://www.wnacg.com/photos-index-aid-325177.html"
                size="large"
                onKeydown={(event: KeyboardEvent) => {
                  if (event.key === 'Enter') {
                    void startDownload()
                  }
                }}
              />
              <NButton type="primary" size="large" loading={submitting.value} onClick={() => void startDownload()}>
                开始下载
              </NButton>
            </div>
          </div>

          <div class="mb-4 flex items-center justify-between">
            <div class="text-xl font-700">任务列表</div>
            <div class="text-sm text-slate-500">每 2 秒自动刷新</div>
          </div>

          {sortedTasks.value.length === 0 ? (
            <NCard bordered={false} class="rounded-6 shadow-[0_10px_40px_rgba(15,23,42,0.06)]">
              <NEmpty description="还没有下载任务" />
            </NCard>
          ) : (
            <div class="space-y-8">
              {(['downloading', 'success', 'failed'] as const).map((group) =>
                groupedTasks.value[group].length > 0 ? (
                  <section key={group}>
                    <div class="mb-3 flex items-center gap-3">
                      <div class="text-lg font-700">{statusLabel(group)}</div>
                      <NTag bordered={false} type={statusType(group) as 'success' | 'error' | 'warning'}>
                        {groupedTasks.value[group].length}
                      </NTag>
                    </div>
                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                      {groupedTasks.value[group].map((task) => (
                        <NCard
                          key={task.id}
                          bordered={false}
                          class="overflow-hidden rounded-6 border border-solid border-slate-200 shadow-[0_18px_50px_rgba(15,23,42,0.08)]">
                          <div class="mb-4 flex gap-4">
                            <div class="h-36 w-25 shrink-0 overflow-hidden rounded-4 bg-slate-100">
                              {task.cover ? (
                                <NImage
                                  src={task.cover}
                                  alt={task.title || task.target}
                                  preview-disabled
                                  class="h-full w-full object-cover"
                                />
                              ) : (
                                <div class="flex h-full items-center justify-center text-xs text-slate-400">无封面</div>
                              )}
                            </div>
                            <div class="min-w-0 flex-1">
                              <NSpace align="center" justify="space-between">
                                <NTag type={statusType(task.status) as 'success' | 'error' | 'warning'} bordered={false}>
                                  {statusLabel(task.status)}
                                </NTag>
                                <div class="text-xs text-slate-400">#{task.id.slice(0, 8)}</div>
                              </NSpace>
                              <div class="mt-3 line-clamp-3 text-base font-700 leading-snug">
                                {task.title || task.target}
                              </div>
                              <div class="mt-2 text-sm text-slate-500">
                                已下载 {task.completedPages}
                                {task.totalPages ? ` / ${task.totalPages}` : ''} 页
                              </div>
                            </div>
                          </div>

                          <NProgress
                            percentage={progressPercentage(task)}
                            processing={task.status === 'downloading'}
                            status={
                              task.status === 'failed' ? 'error' : task.status === 'success' ? 'success' : 'info'
                            }
                            show-indicator={false}
                          />

                          {task.error && (
                            <div class="mt-4 rounded-4 bg-rose-50 px-3 py-2 text-sm text-rose-700">{task.error}</div>
                          )}

                          {task.zipPath && (
                            <div class="mt-4 rounded-4 bg-slate-50 px-3 py-2 text-xs text-slate-500 break-all">{task.zipPath}</div>
                          )}
                        </NCard>
                      ))}
                    </div>
                  </section>
                ) : null,
              )}
            </div>
          )}
        </div>
      </div>
    )
  },
})

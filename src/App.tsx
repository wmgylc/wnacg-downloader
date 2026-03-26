import { defineComponent } from 'vue'
import AppContent from './AppContent.tsx'
import WebDownloadDashboard from './WebDownloadDashboard.tsx'
import {
  NConfigProvider,
  NModalProvider,
  NNotificationProvider,
  NMessageProvider,
  GlobalThemeOverrides,
} from 'naive-ui'

export default defineComponent({
  name: 'App',
  setup() {
    const isTauri =
      typeof window !== 'undefined' &&
      (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== undefined

    const themeOverrides: GlobalThemeOverrides = {
      common: {
        primaryColor: '#1677FF',
        primaryColorHover: '#4096FF',
        primaryColorPressed: '#0958D9',
        primaryColorSuppl: '#4096FF',
        borderRadius: '4px',
        borderRadiusSmall: '3px',
        heightMedium: '32px',
      },
      Button: {
        paddingSmall: '0 8px',
        paddingMedium: '0 12px',
      },
      Radio: {
        buttonColorActive: '#1677FF',
        buttonTextColorActive: '#FFF',
      },
      Dropdown: {
        borderRadius: '5px',
        padding: '6px 2px',
        optionColorHover: '#1677FF',
        optionTextColorHover: '#FFF',
        optionHeightMedium: '28px',
      },
    }

    return () => (
      <NConfigProvider theme-overrides={themeOverrides}>
        <NModalProvider>
          <NNotificationProvider placement="bottom-right" max={3}>
            <NMessageProvider>
              {!isTauri && <WebDownloadDashboard />}
              {isTauri && <AppContent />}
            </NMessageProvider>
          </NNotificationProvider>
        </NModalProvider>
      </NConfigProvider>
    )
  },
})

import { defineComponent } from 'vue'
import WebDownloadDashboard from './WebDownloadDashboard.tsx'
import {
  NConfigProvider,
  NModalProvider,
  NNotificationProvider,
  NMessageProvider,
  GlobalThemeOverrides,
} from 'naive-ui'

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

export default defineComponent({
  name: 'App',
  setup() {
    return () => (
      <NConfigProvider theme-overrides={themeOverrides}>
        <NModalProvider>
          <NNotificationProvider placement="bottom-right" max={3}>
            <NMessageProvider>
              <WebDownloadDashboard />
            </NMessageProvider>
          </NNotificationProvider>
        </NModalProvider>
      </NConfigProvider>
    )
  },
})

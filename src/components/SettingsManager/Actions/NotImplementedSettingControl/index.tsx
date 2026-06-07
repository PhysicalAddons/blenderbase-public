import { IAppSetting } from '../../../../models'

type Props = {
    appSetting: IAppSetting
}

const NotImplementedSettingControl = (props: Props) => {
  return (
    <>Setting '{props.appSetting.code}' is not implemented</>
  )
}

export default NotImplementedSettingControl
import React, { ReactNode } from 'react'
import TitleBar from "../Titlebar"
import { Theme } from '@carbon/react'
import { OS_OBJECT } from '../../constants'
import { useThemeStore } from '../../store/themeStore'

type Props = {
	children: ReactNode
}

const MainContainer: React.FC<Props> = ({ children }) => {
	const { resolved } = useThemeStore()
	return (
		<Theme theme={resolved === 'light' ? 'white' : 'g100'} className={OS_OBJECT.win}>
			<TitleBar />
			<main>
				{children}
			</main>
		</Theme>
	);
}

export default MainContainer
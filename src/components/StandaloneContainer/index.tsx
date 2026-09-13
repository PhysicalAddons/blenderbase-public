import { Theme } from '@carbon/react';
import { useThemeStore } from '../../store/themeStore';
import React, { ReactNode } from 'react'
import { OS_OBJECT } from '../../constants';
import StandaloneTitlebar from '../StandaloneTitlebar';

type Props = {
    children: ReactNode
}

const StandaloneContainer: React.FC<Props> = ({ children }) => {
    return (
        <Theme theme={useThemeStore((t) => t.resolved) === 'light' ? 'white' : 'g100'} className={OS_OBJECT.win}>
            <StandaloneTitlebar />
            <main>
                {children}
            </main>
        </Theme>
    );
}

export default StandaloneContainer
import { useEffect } from 'react';
import { IBlenderSeries } from '../../../models';
import SeriesSection from '../SeriesSection';
import { useBlendFileStore } from '../../../store/blendFileStore';
import { Folder } from '@carbon/react/icons';

const Sections = () => {
    const { blenderSeries, setBlenderSeries } = useBlendFileStore()

    useEffect(() => {
        async function init() {
            setBlenderSeries();
        }
        init();
    }, [])

    return (
        <>
            {blenderSeries.length === 0
                ?
                <div
                    className='recent_files__empty_all_blenders'
                >
                    <Folder />
                    <p>No recent files found</p>
                </div>
                :
                <div>
                    {blenderSeries.map((entry: IBlenderSeries) => (
                        <SeriesSection key={entry.id} blenderSeries={entry} />
                    ))}
                </div>
            }
        </>
    )
}

export default Sections
import { IBlenderSeries } from '../../../models';
import SeriesSection from '../SeriesSection';
import { useBlendFileStore } from '../../../store/blendFileStore';
import { Folder } from '@carbon/react/icons';

/** The series list is loaded by the panel (RecentFiles); this only renders it. */
const Sections = () => {
    const blenderSeries = useBlendFileStore((s) => s.blenderSeries)

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
                        <SeriesSection key={entry.id} blenderSeriesId={entry.id} />
                    ))}
                </div>
            }
        </>
    )
}

export default Sections

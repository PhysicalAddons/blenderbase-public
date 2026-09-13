import { useEffect } from 'react';
import { IBlenderVersion, IBlendFile } from '../../../models'
import { ContainedList, ContainedListItem, OverflowMenu, OverflowMenuItem } from '@carbon/react';
import { useShallow } from 'zustand/react/shallow';
import { BlendFileService } from '../../../services/blendFileService';
import { ChevronDown } from '@carbon/react/icons';
import { useBlenderManagerStore } from '../../../store/blenderManagerStore';
import EmptyRecentFilesBurgerMenu from '../EmptyRecentFilesBurgerMenu';
import { parseVersion } from '../../../utility';
import { useBlendFileStore } from '../../../store/blendFileStore';
import { postStatus, postStatusError } from '../../../store/statusStore';

type Props = {
    blenderSeriesId: string,
}

const blendFileService = new BlendFileService();

const SeriesSection = (props: Props) => {
    const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
    const blenderSeries = useBlendFileStore((s) => s.blenderSeries.find((x) => x.id === props.blenderSeriesId))
    const blendFiles = useBlendFileStore((s) => s.blendFilesBySeries[props.blenderSeriesId])
    const { setBlendFiles, setSeriesCollapsed } = useBlendFileStore(
        useShallow((s) => ({ setBlendFiles: s.setBlendFiles, setSeriesCollapsed: s.setSeriesCollapsed }))
    )

    useEffect(() => {
        // The store owns the result, so nothing is set on this component after unmount.
        setBlendFiles(props.blenderSeriesId).catch((e) => console.error(e));
    }, [props.blenderSeriesId])

    if (!blenderSeries) {
        return null;
    }
    const files: IBlendFile[] = blendFiles ?? [];

    const toggleCollapsed = async (): Promise<void> => {
        await setSeriesCollapsed(blenderSeries.id, !blenderSeries.is_collapsed);
    }

    const openBlendFile = async (blendFile: IBlendFile, blenderVersion: IBlenderVersion | undefined): Promise<void> => {
        if (!blenderVersion) {
            postStatusError(`No Blender ${blenderSeries.series} version is installed to open ${blendFile.file_name}`);
            return;
        }
        postStatus(`Opening ${blendFile.file_name} in Blender ${blenderVersion.version}…`, true);
        try {
            await blendFileService.openBlendFile(blendFile.id, blenderVersion.id);
            postStatus(`Opened ${blendFile.file_name} in Blender ${blenderVersion.version}`);
        } catch (e) {
            console.error(e);
            postStatusError(`Opening ${blendFile.file_name} failed: ${e}`);
        }
        setBlendFiles(props.blenderSeriesId).catch((e) => console.error(e));
    }

    /** Opens the file with the newest installed version of this file's own series. */
    const openBlendFileInSeriesBlender = async (blendFile: IBlendFile): Promise<void> => {
        const bv = installedBuilds
            .filter(x => x.series === blenderSeries.series)
            .reduce((best, current) => {
                if (!best) return current;
                return parseVersion(current.version!) > parseVersion(best.version!)
                    ? current
                    : best;
            }, undefined as IBlenderVersion | undefined);
        await openBlendFile(blendFile, bv);
    }

    return (
        <div>
            <ContainedList
                size="sm"
                isInset={true}
                className="subTitle"
                label={
                    <div
                        className='blender_series_heading'
                        onClick={toggleCollapsed}
                    >
                        <div className='recent_files__blender_series_number'>
                            <span className='blender_series_number'>
                                {blenderSeries.series}
                            </span>
                            {/* One chevron, rotated: right when collapsed, down when expanded. */}
                            <span
                                className={`expand_blend_file_list_icon ${blenderSeries.is_collapsed ? 'expand_blend_file_list_icon--collapsed' : ''}`}
                                title={blenderSeries.is_collapsed ? "Show files" : "Hide files"}
                            >
                                <ChevronDown />
                            </span>
                        </div>
                    </div>
                }
            >
                {!blenderSeries.is_collapsed && (
                    <>
                        {files.map((blendFile) => (
                            <ContainedListItem
                                className="truncate blend_file_blender_version_menu"
                                key={blendFile.id}
                                disabled={false}
                                onClick={() => openBlendFileInSeriesBlender(blendFile)}
                                action={
                                    installedBuilds.length === 0 ?
                                        <EmptyRecentFilesBurgerMenu />
                                        :
                                        <OverflowMenu
                                            aria-label="overflow-menu"
                                            flipped={true}
                                            size="sm"
                                        >
                                            {installedBuilds.map((blenderVersion) => (
                                                <OverflowMenuItem
                                                    key={blenderVersion.id}
                                                    requireTitle={true}
                                                    title={`Open '${blendFile.file_name}' in Blender ${blenderVersion.version} ${blenderVersion.risk_id}`}
                                                    itemText={`${blenderVersion.version} ${blenderVersion.risk_id}`}
                                                    onClick={() => openBlendFile(blendFile, blenderVersion)}
                                                />
                                            ))}
                                        </OverflowMenu>
                                }
                            >
                                <span
                                    title={
`
File path: ${blendFile.file_path}
File size: ${Math.round(blendFile.file_size / 1024)} KB
Date created: ${blendFile.created_datetime}
Date modified: ${blendFile.modified_datetime}
Date accessed: ${blendFile.accessed_datetime}
`
                                    }
                                >
                                    {blendFile.file_name}
                                </span>
                            </ContainedListItem>
                        ))}
                    </>
                )}
            </ContainedList>
            {/* Keeps the next series header on the 56px row grid when a group has an odd number of files. */}
            {!blenderSeries.is_collapsed && files.length % 2 === 1 && (
                <div className="recent_files__spacer" />
            )}
        </div>
    )
}

export default SeriesSection

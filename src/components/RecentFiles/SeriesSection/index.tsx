import { useEffect, useState } from 'react';
import { IBlenderSeries, IBlenderVersion, IBlendFile } from '../../../models'
import { ContainedList, ContainedListItem, OverflowMenu, OverflowMenuItem } from '@carbon/react';
import { BlendFileService } from '../../../services/blendFileService';
import { ChevronDown, ChevronUp } from '@carbon/react/icons';
import { useBlenderManagerStore } from '../../../store/blenderManagerStore';
import EmptyRecentFilesBurgerMenu from '../EmptyRecentFilesBurgerMenu';
import { parseVersion } from '../../../utility';
import { useBlendFileStore } from '../../../store/blendFileStore';

type Props = {
    blenderSeries: IBlenderSeries,
}

const blendFileService = new BlendFileService();

const SeriesSection = (props: Props) => {
    const [localBlenderSeries, setLocalBlenderSeries] = useState<IBlenderSeries>(props.blenderSeries);
    const { installedBuilds } = useBlenderManagerStore()
    const { blendFiles, setBlendFiles } = useBlendFileStore()
    useEffect(() => {
        async function init() {
            setBlendFiles(props.blenderSeries.id);
        }
        init();
    }, [])
    const updateSeries = async (): Promise<void> => {
        try {
            const newBlenderSeries = {
                ...localBlenderSeries,
                is_collapsed: !localBlenderSeries.is_collapsed,
            };
            await blendFileService.updateBlenderSeries(newBlenderSeries);
            setLocalBlenderSeries(newBlenderSeries);
        } catch (e) {
            console.error(e);
        }
    }
    const openBlendFile = async (blendFile: IBlendFile, blenderVersion: IBlenderVersion | undefined): Promise<void> => {
        try {
            if (blenderVersion) {
                await blendFileService.openBlendFile(blendFile, blenderVersion);
            }
        } catch (e) {
            console.error(e);
        } finally {
            setBlendFiles(props.blenderSeries.id);
        }
    }
    const openBlendFileInSeriesBlender = async (blendFile: IBlendFile): Promise<void> => {
        try {
            const bv = installedBuilds
                .filter(x => x.series === localBlenderSeries.series)
                .reduce((best, current) => {
                    if (!best) return current;
                    return parseVersion(current.version!) > parseVersion(best.version!)
                        ? current
                        : best;
                }, undefined as IBlenderVersion | undefined);
            if (!bv) {
                console.error(`No Blender version of series ${localBlenderSeries.series}`);
            } 
            await openBlendFile(blendFile, bv);
        } catch (e) {
            console.error(e);
        } finally {
            setBlendFiles(props.blenderSeries.id);
        }
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
                        onClick={async () => updateSeries()}
                    >
                        <div className='recent_files__blender_series_number'>
                            <span className='blender_series_number'>
                                {localBlenderSeries.series}
                            </span>
                            {localBlenderSeries.is_collapsed === true ?
                                <span
                                    className='expand_blend_file_list_icon'
                                >
                                    <ChevronDown />
                                </span>
                                :
                                <span
                                    className='expand_blend_file_list_icon'
                                >
                                    <ChevronUp />
                                </span>
                            }
                        </div>
                    </div>
                }
            >
                {!localBlenderSeries.is_collapsed && (
                    <>
                        {blendFiles.map((blendFile, index) => (
                            <ContainedListItem
                                className="truncate blend_file_blender_version_menu"
                                key={index}
                                disabled={false}
                                onClick={async () => await openBlendFileInSeriesBlender(blendFile)} // * Ok
                                action={
                                    installedBuilds.length === 0 ?
                                        <EmptyRecentFilesBurgerMenu />
                                        :
                                        <OverflowMenu
                                            aria-label="overflow-menu"
                                            flipped={true}
                                            // theme="g100" 
                                            size="sm"
                                        >
                                            {installedBuilds.map((blenderVersion, id) => (
                                                <OverflowMenuItem
                                                    key={id}
                                                    requireTitle={true}
                                                    title={`Open '${blendFile.file_name}' in Blender ${blenderVersion.version} ${blenderVersion.risk_id}`}
                                                    itemText={`${blenderVersion.version} ${blenderVersion.risk_id}`} 
                                                    onClick={async () => await openBlendFile(blendFile, blenderVersion)} // * Ok
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
        </div>
    )
}

export default SeriesSection
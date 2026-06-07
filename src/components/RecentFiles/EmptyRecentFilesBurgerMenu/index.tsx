import { OverflowMenu, OverflowMenuItem } from "@carbon/react"

const EmptyRecentFilesBurgerMenu = () => {
    return (
        <OverflowMenu aria-label="overflow-menu" flipped={true} size="sm">
            <OverflowMenuItem
                requireTitle={true}
                title="No versions"
                key="empty_recent_files_burger_menu"
                itemText="No versions found"
                disabled={false}
            />
        </OverflowMenu>
    )
}

export default EmptyRecentFilesBurgerMenu

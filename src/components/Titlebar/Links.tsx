import { NavLink } from "react-router-dom"

const Links = () => {
	return (
		<div
			className="navigation_links nav-cell"
		>
			<NavLink
				className={"navigation_links_options"}
				title="Library tab"
				to="/library"
			>
				Library
			</NavLink>
			<NavLink
				className={"navigation_links_options"}
				title="Settings tab"
				to="/settings"
			>
				Settings
			</NavLink>
		</div>
	)
}

export default Links
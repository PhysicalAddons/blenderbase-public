import { useRef } from 'react';
import { Information } from '@carbon/react/icons';
import { postStatus, useStatusStore } from '../../store/statusStore';

type DocumentationLinkProps = {
	/** The wiki page to open in the browser. */
	href: string,
	/** What the status line says while the button is hovered, e.g. "How Settings work · opens the documentation in your browser". */
	hint: string,
}

/**
 * The small info button next to a column title that opens the documentation for that view.
 * It explains itself in the status line, like the title-bar buttons; what was shown before the
 * hover comes back on leave, unless something else posted meanwhile.
 */
const DocumentationLink = ({ href, hint }: DocumentationLinkProps) => {
	const statusBeforeHint = useRef<{ message: string, isBusy: boolean, isError: boolean } | null>(null);
	const showHint = () => {
		const s = useStatusStore.getState();
		if (s.isBusy) {
			return;
		}
		statusBeforeHint.current = { message: s.message, isBusy: s.isBusy, isError: s.isError };
		postStatus(hint);
	};
	const hideHint = () => {
		const before = statusBeforeHint.current;
		statusBeforeHint.current = null;
		if (before && useStatusStore.getState().message === hint) {
			useStatusStore.getState().setStatus(before.message, before.isBusy, before.isError);
		}
	};
	return (
		<a
			className='column_header__info'
			href={href}
			target='_blank'
			rel='noopener'
			aria-label={`${hint} (opens the documentation in your browser)`}
			onMouseEnter={showHint}
			onMouseLeave={hideHint}
			onFocus={showHint}
			onBlur={hideHint}
		>
			<Information size={20} />
		</a>
	);
};

export default DocumentationLink

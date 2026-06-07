import { Routes, Route } from 'react-router-dom';
import MainContainer from "./components/MainContainer";
import Library from './views/Library';
import Settings from './views/Settings';
import DownloadFilePopup from './standalone/DownloadFilePopup';
import StandaloneContainer from './components/StandaloneContainer';

const AppRouter = () => {
    return (
        <Routes>
            {/* <Route path="/" element={<MainContainer><div>/</div></MainContainer>} /> */}
            <Route path="/" element={<MainContainer><Library /></MainContainer>} />
            <Route path="/library" element={<MainContainer><Library /></MainContainer>} />
            <Route path="/settings" element={<MainContainer><Settings /></MainContainer>} />
            {/* <Route path="/popup/_popup" element={} /> */}
            <Route path="/standalone/DownloadFilePopup" element={<StandaloneContainer><DownloadFilePopup /></StandaloneContainer>} />
            <Route path="*" element={<h1>404 Error: Page not found.</h1>} />
        </Routes>
    );
};

export default AppRouter;
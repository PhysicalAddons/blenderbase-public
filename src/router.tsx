import { Routes, Route } from 'react-router-dom';
import MainContainer from "./components/MainContainer";
import Home from './views/Home';
import DownloadFilePopup from './standalone/DownloadFilePopup';
import StandaloneContainer from './components/StandaloneContainer';

const AppRouter = () => {
    return (
        <Routes>
            <Route path="/" element={<MainContainer><Home /></MainContainer>} />
            <Route path="/standalone/DownloadFilePopup" element={<StandaloneContainer><DownloadFilePopup /></StandaloneContainer>} />
            <Route path="*" element={<h1>404 Error: Page not found.</h1>} />
        </Routes>
    );
};

export default AppRouter;

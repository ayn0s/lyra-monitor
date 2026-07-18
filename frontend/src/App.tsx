import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppShell from "./components/layout/AppShell";
import AgentLayout from "./components/layout/AgentLayout";
import DashboardPage from "./pages/DashboardPage";
import AgentMetricsPage from "./pages/AgentMetricsPage";
import AgentServicesPage from "./pages/AgentServicesPage";
import AgentTerminalPage from "./pages/AgentTerminalPage";
import NotFoundPage from "./pages/NotFoundPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<DashboardPage />} />
          <Route path="agents/:addr" element={<AgentLayout />}>
            <Route index element={<Navigate to="metrics" replace />} />
            <Route path="metrics" element={<AgentMetricsPage />} />
            <Route path="services" element={<AgentServicesPage />} />
            <Route path="terminal" element={<AgentTerminalPage />} />
          </Route>
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

/**
 * VKS ECMS — ModuleSlot
 *
 * Component container: hien fallback khi module tat, load module khi bat.
 * Tu dong handle loading state va error boundary.
 */

import React, { Suspense } from "react";
import { useModuleConfig } from "./useModuleConfig";

interface ModuleSlotProps {
  moduleId: string;
  fallback: React.ReactNode;
  /** Module component loaders indexed by version */
  loaders?: Record<string, React.LazyExoticComponent<React.ComponentType<any>>>;
  /** Props forwarded to the module component */
  moduleProps?: Record<string, unknown>;
}

function ModuleLoadingSpinner() {
  return (
    <div className="module-loading">
      <div className="module-spinner" />
      <span>Đang tải module...</span>
    </div>
  );
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ModuleErrorBoundary extends React.Component<
  { fallback: React.ReactNode; children: React.ReactNode },
  ErrorBoundaryState
> {
  constructor(props: { fallback: React.ReactNode; children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="module-error">
          <p>⚠️ Module lỗi: {this.state.error?.message}</p>
          <p>Đang hiện chế độ cơ bản.</p>
          {this.props.fallback}
        </div>
      );
    }
    return this.props.children;
  }
}

export function ModuleSlot({
  moduleId,
  fallback,
  loaders,
  moduleProps,
}: ModuleSlotProps) {
  const { enabled, version, loading } = useModuleConfig(moduleId);

  // Still loading configs
  if (loading) {
    return <ModuleLoadingSpinner />;
  }

  // Module disabled → show fallback
  if (!enabled || !loaders) {
    return <>{fallback}</>;
  }

  // Find the right version loader
  const LazyComponent = loaders[version];
  if (!LazyComponent) {
    return <>{fallback}</>;
  }

  return (
    <ModuleErrorBoundary fallback={fallback}>
      <Suspense fallback={<ModuleLoadingSpinner />}>
        <LazyComponent {...(moduleProps ?? {})} />
      </Suspense>
    </ModuleErrorBoundary>
  );
}

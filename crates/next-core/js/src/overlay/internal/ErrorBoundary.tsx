import * as React from "react";

type InternalErrorBoundaryProps = {
  error: Error | null;
  onError: (error: Error, componentStack: string | null) => void;
  fallback: React.ReactNode | null;
  children?: React.ReactNode;
};

type ErrorBoundaryState = { error: Error | null };

class InternalErrorBoundary extends React.PureComponent<
  InternalErrorBoundaryProps,
  ErrorBoundaryState
> {
  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  static getDerivedStateFromProps(
    nextProps: Readonly<InternalErrorBoundaryProps>,
    prevState: ErrorBoundaryState
  ) {
    if (nextProps.error === prevState.error) {
      return null;
    }

    return {
      error: nextProps.error,
    };
  }

  state = { error: null };

  componentDidCatch(
    error: Error,
    errorInfo?: { componentStack?: string | null }
  ) {
    this.props.onError(error, errorInfo?.componentStack ?? null);
  }

  render() {
    const { error, fallback, children } = this.props;

    // The component has to be unmounted or else it would continue to error
    if (error != null) {
      return fallback;
    }

    return children;
  }
}

type ErrorBoundaryProps = {
  fallback: React.ReactNode | null;
  children?: React.ReactNode;
};

export function useErrorBoundary(
  onError?: (error: Error, componentStack: string | null) => void
): [Error | null, () => void, React.FunctionComponent<ErrorBoundaryProps>] {
  const [error, setError] = React.useState<Error | null>(null);

  const errorCallback = React.useCallback(
    (error: Error, componentStack: string | null) => {
      setError(error);
      onError && onError(error, componentStack);
    },
    [setError, onError]
  );

  const resetError = React.useCallback(() => {
    setError(null);
  }, [setError]);

  const ErrorBoundary = React.useCallback(
    ({ children, ...props }: ErrorBoundaryProps) => (
      <InternalErrorBoundary {...props} error={error} onError={errorCallback}>
        {children}
      </InternalErrorBoundary>
    ),
    [error, errorCallback]
  );

  return [error, resetError, ErrorBoundary];
}

import React, { Component, ErrorInfo, ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";

interface Props {
    children?: ReactNode;
    fallback?: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error("Uncaught error:", error, errorInfo);
    }

    private handleReset = () => {
        this.setState({ hasError: false, error: null });
        window.location.reload();
    };

    public render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <div className="flex min-h-screen flex-col items-center justify-center bg-background p-6 text-center">
                    <div className="mb-6 rounded-full bg-status-critical/10 p-4 text-status-critical">
                        <AlertTriangle className="h-12 w-12" />
                    </div>
                    <h1 className="mb-2 text-2xl font-bold tracking-tight">
                        Something went wrong
                    </h1>
                    <p className="mb-8 max-w-md text-muted-foreground">
                        An unexpected error occurred. We've been notified and are looking into it.
                    </p>
                    <div className="flex flex-col gap-2 sm:flex-row">
                        <Button onClick={this.handleReset} className="gap-2">
                            <RefreshCw className="h-4 w-4" />
                            Try Again
                        </Button>
                        <Button variant="outline" onClick={() => window.location.href = "/"}>
                            Return Home
                        </Button>
                    </div>
                    {process.env.NODE_ENV === "development" && this.state.error && (
                        <pre className="mt-8 max-w-full overflow-auto rounded-lg bg-muted p-4 text-left text-xs text-muted-foreground">
                            {this.state.error.toString()}
                        </pre>
                    )}
                </div>
            );
        }

        return this.props.children;
    }
}

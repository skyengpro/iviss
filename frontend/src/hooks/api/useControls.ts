import { useGetControls } from "@/openapi-rq/queries/queries";
import { GetControlsData } from "@/openapi-rq/requests/types.gen";

export const useControls = (params?: GetControlsData) => {
    const { data, isLoading, refetch } = useGetControls(params);
    return {
        controls: data,
        isLoading,
        refetch
    };
};

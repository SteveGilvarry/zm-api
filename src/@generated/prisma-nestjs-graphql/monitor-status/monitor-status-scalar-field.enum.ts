import { registerEnumType } from '@nestjs/graphql';

export enum Monitor_StatusScalarFieldEnum {
    MonitorId = "MonitorId",
    Status = "Status",
    CaptureFPS = "CaptureFPS",
    AnalysisFPS = "AnalysisFPS",
    CaptureBandwidth = "CaptureBandwidth",
    DayEventDiskSpace = "DayEventDiskSpace"
}


registerEnumType(Monitor_StatusScalarFieldEnum, { name: 'Monitor_StatusScalarFieldEnum', description: undefined })

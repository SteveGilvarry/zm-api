import { registerEnumType } from '@nestjs/graphql';

export enum ControlPresetsScalarFieldEnum {
    MonitorId = "MonitorId",
    Preset = "Preset",
    Label = "Label"
}


registerEnumType(ControlPresetsScalarFieldEnum, { name: 'ControlPresetsScalarFieldEnum', description: undefined })

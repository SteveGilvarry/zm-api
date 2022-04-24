import { registerEnumType } from '@nestjs/graphql';

export enum TriggersX10ScalarFieldEnum {
    MonitorId = "MonitorId",
    Activation = "Activation",
    AlarmInput = "AlarmInput",
    AlarmOutput = "AlarmOutput"
}


registerEnumType(TriggersX10ScalarFieldEnum, { name: 'TriggersX10ScalarFieldEnum', description: undefined })

import { registerEnumType } from '@nestjs/graphql';

export enum Groups_MonitorsScalarFieldEnum {
    Id = "Id",
    GroupId = "GroupId",
    MonitorId = "MonitorId"
}


registerEnumType(Groups_MonitorsScalarFieldEnum, { name: 'Groups_MonitorsScalarFieldEnum', description: undefined })

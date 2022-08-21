import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusUpdateInput } from '../monitor-status/monitor-status-update.input';
import { Type } from 'class-transformer';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';

@ArgsType()
export class UpdateOneMonitorStatusArgs {

    @Field(() => Monitor_StatusUpdateInput, {nullable:false})
    @Type(() => Monitor_StatusUpdateInput)
    data!: Monitor_StatusUpdateInput;

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:false})
    @Type(() => Monitor_StatusWhereUniqueInput)
    where!: Monitor_StatusWhereUniqueInput;
}

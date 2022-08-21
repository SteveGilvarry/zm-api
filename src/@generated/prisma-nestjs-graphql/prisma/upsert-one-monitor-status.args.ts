import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';
import { Type } from 'class-transformer';
import { Monitor_StatusCreateInput } from '../monitor-status/monitor-status-create.input';
import { Monitor_StatusUpdateInput } from '../monitor-status/monitor-status-update.input';

@ArgsType()
export class UpsertOneMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:false})
    @Type(() => Monitor_StatusWhereUniqueInput)
    where!: Monitor_StatusWhereUniqueInput;

    @Field(() => Monitor_StatusCreateInput, {nullable:false})
    @Type(() => Monitor_StatusCreateInput)
    create!: Monitor_StatusCreateInput;

    @Field(() => Monitor_StatusUpdateInput, {nullable:false})
    @Type(() => Monitor_StatusUpdateInput)
    update!: Monitor_StatusUpdateInput;
}

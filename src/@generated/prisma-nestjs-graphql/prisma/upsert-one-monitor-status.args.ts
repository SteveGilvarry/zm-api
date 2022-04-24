import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';
import { Monitor_StatusCreateInput } from '../monitor-status/monitor-status-create.input';
import { Monitor_StatusUpdateInput } from '../monitor-status/monitor-status-update.input';

@ArgsType()
export class UpsertOneMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:false})
    where!: Monitor_StatusWhereUniqueInput;

    @Field(() => Monitor_StatusCreateInput, {nullable:false})
    create!: Monitor_StatusCreateInput;

    @Field(() => Monitor_StatusUpdateInput, {nullable:false})
    update!: Monitor_StatusUpdateInput;
}

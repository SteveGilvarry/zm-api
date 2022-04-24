import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';
import { MonitorPresetsCreateInput } from './monitor-presets-create.input';
import { MonitorPresetsUpdateInput } from './monitor-presets-update.input';

@ArgsType()
export class UpsertOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    where!: MonitorPresetsWhereUniqueInput;

    @Field(() => MonitorPresetsCreateInput, {nullable:false})
    create!: MonitorPresetsCreateInput;

    @Field(() => MonitorPresetsUpdateInput, {nullable:false})
    update!: MonitorPresetsUpdateInput;
}

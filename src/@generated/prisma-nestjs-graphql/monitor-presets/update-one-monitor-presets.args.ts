import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsUpdateInput } from './monitor-presets-update.input';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';

@ArgsType()
export class UpdateOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsUpdateInput, {nullable:false})
    data!: MonitorPresetsUpdateInput;

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    where!: MonitorPresetsWhereUniqueInput;
}

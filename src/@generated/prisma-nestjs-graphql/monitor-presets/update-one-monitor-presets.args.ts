import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsUpdateInput } from './monitor-presets-update.input';
import { Type } from 'class-transformer';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';

@ArgsType()
export class UpdateOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsUpdateInput, {nullable:false})
    @Type(() => MonitorPresetsUpdateInput)
    data!: MonitorPresetsUpdateInput;

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorPresetsWhereUniqueInput)
    where!: MonitorPresetsWhereUniqueInput;
}

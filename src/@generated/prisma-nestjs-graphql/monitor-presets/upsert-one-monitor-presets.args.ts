import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';
import { Type } from 'class-transformer';
import { MonitorPresetsCreateInput } from './monitor-presets-create.input';
import { MonitorPresetsUpdateInput } from './monitor-presets-update.input';

@ArgsType()
export class UpsertOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorPresetsWhereUniqueInput)
    where!: MonitorPresetsWhereUniqueInput;

    @Field(() => MonitorPresetsCreateInput, {nullable:false})
    @Type(() => MonitorPresetsCreateInput)
    create!: MonitorPresetsCreateInput;

    @Field(() => MonitorPresetsUpdateInput, {nullable:false})
    @Type(() => MonitorPresetsUpdateInput)
    update!: MonitorPresetsUpdateInput;
}

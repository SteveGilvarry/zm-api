import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorPresetsWhereUniqueInput)
    where!: MonitorPresetsWhereUniqueInput;
}

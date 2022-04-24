import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';

@ArgsType()
export class DeleteOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:false})
    where!: MonitorPresetsWhereUniqueInput;
}

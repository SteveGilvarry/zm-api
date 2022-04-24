import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';

@ArgsType()
export class DeleteManyMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    where?: MonitorPresetsWhereInput;
}

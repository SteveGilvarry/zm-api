import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    where?: MonitorPresetsWhereInput;
}

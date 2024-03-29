import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsCreateInput } from './monitor-presets-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsCreateInput, {nullable:false})
    @Type(() => MonitorPresetsCreateInput)
    data!: MonitorPresetsCreateInput;
}

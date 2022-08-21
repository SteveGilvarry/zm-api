import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsCreateManyInput } from './monitor-presets-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyMonitorPresetsArgs {

    @Field(() => [MonitorPresetsCreateManyInput], {nullable:false})
    @Type(() => MonitorPresetsCreateManyInput)
    data!: Array<MonitorPresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}

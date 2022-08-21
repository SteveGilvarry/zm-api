import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusCreateInput } from '../monitor-status/monitor-status-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneMonitorStatusArgs {

    @Field(() => Monitor_StatusCreateInput, {nullable:false})
    @Type(() => Monitor_StatusCreateInput)
    data!: Monitor_StatusCreateInput;
}

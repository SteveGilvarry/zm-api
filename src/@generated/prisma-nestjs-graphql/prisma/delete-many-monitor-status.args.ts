import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    @Type(() => Monitor_StatusWhereInput)
    where?: Monitor_StatusWhereInput;
}

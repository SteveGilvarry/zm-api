import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';

@ArgsType()
export class DeleteManyMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    where?: Monitor_StatusWhereInput;
}

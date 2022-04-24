import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereInput } from './monitors-where.input';

@ArgsType()
export class DeleteManyMonitorsArgs {

    @Field(() => MonitorsWhereInput, {nullable:true})
    where?: MonitorsWhereInput;
}

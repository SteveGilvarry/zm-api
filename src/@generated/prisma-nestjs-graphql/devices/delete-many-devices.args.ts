import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereInput } from './devices-where.input';

@ArgsType()
export class DeleteManyDevicesArgs {

    @Field(() => DevicesWhereInput, {nullable:true})
    where?: DevicesWhereInput;
}

import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereInput } from './devices-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyDevicesArgs {

    @Field(() => DevicesWhereInput, {nullable:true})
    @Type(() => DevicesWhereInput)
    where?: DevicesWhereInput;
}

import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ModelsUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    ManufacturerId?: number;
}

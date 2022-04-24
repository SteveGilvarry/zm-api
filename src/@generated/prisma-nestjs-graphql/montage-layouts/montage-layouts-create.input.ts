import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MontageLayoutsCreateInput {

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:true})
    Positions?: string;
}
